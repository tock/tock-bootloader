//! Tock kernel for the bootloader on nrf52 over CDC/USB.
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE + IEEE 802.15.4 transceiver).

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]
#![feature(const_in_array_repeat_expressions)]

use core::panic::PanicInfo;

use kernel::capabilities;
use kernel::common::dynamic_deferred_call::{DynamicDeferredCall, DynamicDeferredCallClientState};
use kernel::component::Component;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::hil::time::Counter;
use kernel::hil::usb::Client;
use kernel::mpu::MPU;
use kernel::Chip;
#[allow(unused_imports)]
use kernel::{create_capability, debug, debug_gpio, debug_verbose, static_init};

use capsules::virtual_alarm::VirtualMuxAlarm;

use nrf52840::gpio::Pin;

use nrf52::deferred_call_tasks::DeferredCallTask;
use nrf52_components::{self, UartChannel, UartPins};

const LED_KERNEL_PIN: Pin = Pin::P0_13;

const UART_RTS: Option<Pin> = Some(Pin::P0_05);
const UART_TXD: Pin = Pin::P0_06;
const UART_CTS: Option<Pin> = Some(Pin::P0_07);
const UART_RXD: Pin = Pin::P0_08;

include!(concat!(env!("OUT_DIR"), "/attributes.rs"));

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 0;

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840BootloaderPeripherals>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x3000] = [0; 0x3000];

/// Supported drivers by the platform
pub struct Platform {
    bootloader: &'static bootloader::bootloader::Bootloader<
        'static,
        bootloader::uart_receive_multiple_timeout::UartReceiveMultipleTimeout<
            'static,
            VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
        >,
        nrf52::nvmc::Nvmc,
    >,
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, _driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        f(None)
    }
}

pub struct Nrf52840BootloaderPeripherals<'a> {
    pub pwr_clk: nrf52::power::Power<'a>,
    pub rtc: nrf52::rtc::Rtc<'a>,
    pub uarte0: nrf52::uart::Uarte<'a>,
    pub nvmc: nrf52::nvmc::Nvmc,
    pub usbd: nrf52840::usbd::Usbd<'a>,
}

impl<'a> Nrf52840BootloaderPeripherals<'a> {
    pub fn new() -> Self {
        Self {
            pwr_clk: nrf52::power::Power::new(),
            rtc: nrf52::rtc::Rtc::new(),
            uarte0: nrf52::uart::Uarte::new(),
            nvmc: nrf52::nvmc::Nvmc::new(),
            usbd: nrf52::usbd::Usbd::new(),
        }
    }
    // Necessary for setting up circular dependencies
    pub fn init(&'a self) {
        self.pwr_clk.set_usb_client(&self.usbd);
        self.usbd.set_power_ref(&self.pwr_clk);
    }
}
impl<'a> kernel::InterruptService<DeferredCallTask> for Nrf52840BootloaderPeripherals<'a> {
    unsafe fn service_interrupt(&self, interrupt: u32) -> bool {
        match interrupt {
            nrf52::peripheral_interrupts::POWER_CLOCK => self.pwr_clk.handle_interrupt(),
            nrf52::peripheral_interrupts::RTC1 => self.rtc.handle_interrupt(),
            nrf52::peripheral_interrupts::UART0 => self.uarte0.handle_interrupt(),
            nrf52840::peripheral_interrupts::USBD => self.usbd.handle_interrupt(),
            _ => return false,
        }
        true
    }
    unsafe fn service_deferred_call(&self, task: DeferredCallTask) -> bool {
        match task {
            DeferredCallTask::Nvmc => self.nvmc.handle_interrupt(),
        }
        true
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52840::init();
    // Initialize chip peripheral drivers
    let peripherals = static_init!(
        Nrf52840BootloaderPeripherals,
        Nrf52840BootloaderPeripherals::new()
    );

    // set up circular peripheral dependencies
    peripherals.init();

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    //--------------------------------------------------------------------------
    // BOOTLOADER ENTRY
    //--------------------------------------------------------------------------

    // Decide very early if we want to stay in the bootloader so we don't run a
    // bunch of init code just to reset into the kernel.

    let bootloader_entry_mode = static_init!(
        bootloader_nrf52::bootloader_entry_gpregret::BootloaderEntryGpRegRet,
        bootloader_nrf52::bootloader_entry_gpregret::BootloaderEntryGpRegRet::new(
            &peripherals.pwr_clk
        )
    );

    let bootloader_enterer = static_init!(
        bootloader::bootloader::BootloaderEnterer<'static>,
        bootloader::bootloader::BootloaderEnterer::new(bootloader_entry_mode,)
    );

    // First decide if we want to actually run the bootloader or not.
    bootloader_enterer.check();

    //--------------------------------------------------------------------------
    // CAPABILITIES
    //--------------------------------------------------------------------------

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    //--------------------------------------------------------------------------
    // Deferred Call (Dynamic) Setup
    //--------------------------------------------------------------------------

    let dynamic_deferred_call_clients =
        static_init!([DynamicDeferredCallClientState; 2], Default::default());
    let dynamic_deferred_caller = static_init!(
        DynamicDeferredCall,
        DynamicDeferredCall::new(dynamic_deferred_call_clients)
    );
    DynamicDeferredCall::set_global_instance(dynamic_deferred_caller);

    //--------------------------------------------------------------------------
    // ALARM & TIMER
    //--------------------------------------------------------------------------

    let rtc = &peripherals.rtc;
    rtc.start();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_helper!(nrf52::rtc::Rtc));

    //--------------------------------------------------------------------------
    // UART DEBUGGING
    //--------------------------------------------------------------------------

    let channel = nrf52_components::UartChannelComponent::new(
        UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD)),
        mux_alarm,
        &peripherals.uarte0,
    )
    .finalize(());

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux =
        components::console::UartMuxComponent::new(channel, 115200, dynamic_deferred_caller)
            .finalize(());

    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    //--------------------------------------------------------------------------
    // CDC
    //--------------------------------------------------------------------------

    // Setup the CDC-ACM over USB driver that we will use for UART.
    // We use the Arduino Vendor ID and Product ID since the device is the same.

    // Create the strings we include in the USB descriptor. We use the hardcoded
    // DEVICEADDR register on the nRF52 to set the serial number.
    let serial_number_buf = static_init!([u8; 17], [0; 17]);
    let serial_number_string: &'static str =
        nrf52::ficr::FICR_INSTANCE.address_str(serial_number_buf);
    let strings = static_init!(
        [&str; 3],
        [
            "Arduino",              // Manufacturer
            "Nano 33 BLE - TockOS", // Product
            serial_number_string,   // Serial number
        ]
    );

    let cdc = components::cdc::CdcAcmComponent::new(
        &peripherals.usbd,
        capsules::usb::cdc::MAX_CTRL_PACKET_SIZE_NRF52840,
        0x2341,
        0x005a,
        strings,
        mux_alarm,
        dynamic_deferred_caller,
    )
    .finalize(components::usb_cdc_acm_component_helper!(
        nrf52::usbd::Usbd,
        nrf52::rtc::Rtc
    ));

    //--------------------------------------------------------------------------
    // BOOTLOADER
    //--------------------------------------------------------------------------

    let recv_auto_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
        VirtualMuxAlarm::new(mux_alarm)
    );

    let recv_auto_cdc = static_init!(
        bootloader::uart_receive_multiple_timeout::UartReceiveMultipleTimeout<
            'static,
            VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
        >,
        bootloader::uart_receive_multiple_timeout::UartReceiveMultipleTimeout::new(
            cdc,
            recv_auto_virtual_alarm,
            &mut bootloader::uart_receive_multiple_timeout::BUF,
        )
    );
    recv_auto_virtual_alarm.set_alarm_client(recv_auto_cdc);

    let pagebuffer = static_init!(nrf52::nvmc::NrfPage, nrf52::nvmc::NrfPage::default());

    // static mut PAGEBUFFER: nrf52::nvmc::NrfPage = nrf52::nvmc::NrfPage::default();
    let bootloader = static_init!(
        bootloader::bootloader::Bootloader<
            'static,
            bootloader::uart_receive_multiple_timeout::UartReceiveMultipleTimeout<
                'static,
                VirtualMuxAlarm<'static, nrf52::rtc::Rtc>,
            >,
            nrf52::nvmc::Nvmc,
        >,
        bootloader::bootloader::Bootloader::new(
            recv_auto_cdc,
            &peripherals.nvmc,
            pagebuffer,
            &mut bootloader::bootloader::BUF
        )
    );
    hil::uart::Transmit::set_transmit_client(cdc, bootloader);
    hil::uart::Receive::set_receive_client(cdc, recv_auto_cdc);
    hil::uart::Receive::set_receive_client(recv_auto_cdc, bootloader);
    hil::flash::HasClient::set_client(&peripherals.nvmc, bootloader);

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52_components::NrfClockComponent::new().finalize(());

    let platform = Platform { bootloader };

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840BootloaderPeripherals>,
        nrf52840::chip::NRF52::new(peripherals)
    );
    CHIP = Some(chip);

    // Need to disable the MPU because the bootloader seems to set it up.
    chip.mpu().clear_mpu();

    //debug!("Bootloader init");

    // Configure the USB stack to enable a serial port over CDC-ACM.
    cdc.enable();
    cdc.attach();

    // Actually run the bootloader.
    platform.bootloader.start();

    //--------------------------------------------------------------------------
    // MAIN LOOP
    //--------------------------------------------------------------------------

    let scheduler = components::sched::round_robin::RoundRobinComponent::new(&PROCESSES)
        .finalize(components::rr_component_helper!(NUM_PROCS));
    board_kernel.kernel_loop(&platform, chip, None, scheduler, &main_loop_capability);
}

#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    loop {}
}
