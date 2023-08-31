//! Tock kernel for the bootloader on nrf52 over UART.
//!
//! It is based on nRF52840 SoC.

#![no_std]
// Disable this attribute when documenting, as a workaround for
// https://github.com/rust-lang/rust/issues/62184.
#![cfg_attr(not(doc), no_main)]

use core::panic::PanicInfo;

use kernel::capabilities;
use kernel::component::Component;
use kernel::create_capability;
use kernel::hil;
use kernel::hil::time::Alarm;
use kernel::hil::time::Counter;
use kernel::platform::KernelResources;
use kernel::platform::SyscallDriverLookup;
use kernel::static_init;

use bootloader::null_scheduler::NullScheduler;

use capsules_core::virtualizers::virtual_alarm::VirtualMuxAlarm;

use nrf52840::gpio::Pin;
use nrf52840::interrupt_service::Nrf52840DefaultPeripherals;

const UART_TXD: Pin = Pin::P0_24;
const UART_RXD: Pin = Pin::P0_22;

const LED_GREEN: Pin = Pin::P0_13;
#[allow(dead_code)]
const LED_RED: Pin = Pin::P0_14;

const BUTTON_CONFIG: Pin = Pin::P0_25;
#[allow(dead_code)]
const BUTTON_USER: Pin = Pin::P0_23;

include!(concat!(env!("OUT_DIR"), "/attributes.rs"));

// Number of concurrent processes this platform supports.
const NUM_PROCS: usize = 0;

static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>> = None;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

/// Function to allow the bootloader to exit by reseting the chip.
fn bootloader_exit() {
    unsafe {
        cortexm4::scb::reset();
    }
}

/// Supported drivers by the platform
pub struct Platform {
    bootloader: &'static bootloader::bootloader::Bootloader<
        'static,
        bootloader::uart_receive_multiple_timeout::UartReceiveMultipleTimeout<
            'static,
            VirtualMuxAlarm<'static, nrf52::rtc::Rtc<'static>>,
        >,
        bootloader::flash_large_to_small::FlashLargeToSmall<'static, nrf52::nvmc::Nvmc>,
    >,
    scheduler: &'static NullScheduler,
}

impl KernelResources<nrf52840::chip::NRF52<'static, Nrf52840DefaultPeripherals<'static>>>
    for Platform
{
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type CredentialsCheckingPolicy = ();
    type Scheduler = NullScheduler;
    type SchedulerTimer = ();
    type WatchDog = ();
    type ContextSwitchCallback = ();

    fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
        &self
    }
    fn syscall_filter(&self) -> &Self::SyscallFilter {
        &()
    }
    fn process_fault(&self) -> &Self::ProcessFault {
        &()
    }
    fn credentials_checking_policy(&self) -> &'static Self::CredentialsCheckingPolicy {
        &()
    }
    fn scheduler(&self) -> &Self::Scheduler {
        self.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &()
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

impl SyscallDriverLookup for Platform {
    fn with_driver<F, R>(&self, _driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
    {
        f(None)
    }
}

#[inline(never)]
unsafe fn create_peripherals() -> &'static mut Nrf52840DefaultPeripherals<'static> {
    let ieee802154_ack_buf = static_init!(
        [u8; nrf52840::ieee802154_radio::ACK_BUF_SIZE],
        [0; nrf52840::ieee802154_radio::ACK_BUF_SIZE]
    );
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = static_init!(
        Nrf52840DefaultPeripherals,
        Nrf52840DefaultPeripherals::new(ieee802154_ack_buf)
    );

    nrf52840_peripherals
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn main() {
    // Loads relocations and clears BSS
    nrf52840::init();
    // Initialize chip peripheral drivers
    let nrf52840_peripherals = create_peripherals();

    // set up circular peripheral dependencies
    nrf52840_peripherals.init();
    let base_peripherals = &nrf52840_peripherals.nrf52;

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    //--------------------------------------------------------------------------
    // BOOTLOADER ENTRY
    //--------------------------------------------------------------------------

    // Decide very early if we want to stay in the bootloader so we don't run a
    // bunch of init code just to reset into the kernel.

    let bootloader_entry_mode = static_init!(
        bootloader::bootloader_entry_gpio::BootloaderEntryGpio<nrf52840::gpio::GPIOPin>,
        bootloader::bootloader_entry_gpio::BootloaderEntryGpio::new(
            &nrf52840_peripherals.gpio_port[BUTTON_CONFIG]
        )
    );

    let bootloader_jumper = static_init!(
        bootloader_cortexm::jumper::CortexMJumper,
        bootloader_cortexm::jumper::CortexMJumper::new()
    );

    let active_notifier_led = static_init!(
        kernel::hil::led::LedLow<'static, nrf52840::gpio::GPIOPin>,
        kernel::hil::led::LedLow::new(&nrf52840_peripherals.gpio_port[LED_GREEN])
    );

    let bootloader_active_notifier = static_init!(
        bootloader::active_notifier_ledon::ActiveNotifierLedon,
        bootloader::active_notifier_ledon::ActiveNotifierLedon::new(active_notifier_led)
    );

    let bootloader_enterer = static_init!(
        bootloader::bootloader::BootloaderEnterer<'static>,
        bootloader::bootloader::BootloaderEnterer::new(
            bootloader_entry_mode,
            bootloader_jumper,
            bootloader_active_notifier
        )
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
    // ALARM & TIMER
    //--------------------------------------------------------------------------

    let rtc = &base_peripherals.rtc;
    let _ = rtc.start();

    let mux_alarm = components::alarm::AlarmMuxComponent::new(rtc)
        .finalize(components::alarm_mux_component_static!(nrf52::rtc::Rtc));

    // //--------------------------------------------------------------------------
    // // UART DEBUGGING
    // //--------------------------------------------------------------------------

    // let channel = nrf52_components::UartChannelComponent::new(
    //     UartChannel::Pins(UartPins::new(UART_RTS, UART_TXD, UART_CTS, UART_RXD)),
    //     mux_alarm,
    //     &base_peripherals.uarte0,
    // )
    // .finalize(());

    // // Create a shared UART channel for the console and for kernel debug.
    // let uart_mux =
    //     components::console::UartMuxComponent::new(channel, 115200, dynamic_deferred_caller)
    //         .finalize(());

    // // Create the debugger object that handles calls to `debug!()`.
    // components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    //--------------------------------------------------------------------------
    // SCHEDULER
    //--------------------------------------------------------------------------

    let null_scheduler = static_init!(NullScheduler, NullScheduler::new());

    //--------------------------------------------------------------------------
    // BOOTLOADER
    //--------------------------------------------------------------------------

    // Setup receive with timeout.
    let recv_auto_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    recv_auto_virtual_alarm.setup();

    let recv_auto_uart = static_init!(
        bootloader::uart_receive_multiple_timeout::UartReceiveMultipleTimeout<
            'static,
            VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
        >,
        bootloader::uart_receive_multiple_timeout::UartReceiveMultipleTimeout::new(
            &base_peripherals.uarte0,
            recv_auto_virtual_alarm,
            &mut bootloader::uart_receive_multiple_timeout::BUF
        )
    );
    recv_auto_virtual_alarm.set_alarm_client(recv_auto_uart);

    // Setup the UART pins
    let _ = base_peripherals.uarte0.initialize(
        nrf52840::pinmux::Pinmux::new(UART_TXD as u32),
        nrf52840::pinmux::Pinmux::new(UART_RXD as u32),
        None,
        None,
    );

    let nrfpagebuffer = static_init!(nrf52::nvmc::NrfPage, nrf52::nvmc::NrfPage::default());

    let flash_adapter = static_init!(
        bootloader::flash_large_to_small::FlashLargeToSmall<'static, nrf52840::nvmc::Nvmc>,
        bootloader::flash_large_to_small::FlashLargeToSmall::new(
            &base_peripherals.nvmc,
            nrfpagebuffer,
        )
    );
    hil::flash::HasClient::set_client(&base_peripherals.nvmc, flash_adapter);

    let pagebuffer = static_init!(
        bootloader::flash_large_to_small::FiveTwelvePage,
        bootloader::flash_large_to_small::FiveTwelvePage::default()
    );

    let bootloader = static_init!(
        bootloader::bootloader::Bootloader<
            'static,
            bootloader::uart_receive_multiple_timeout::UartReceiveMultipleTimeout<
                'static,
                VirtualMuxAlarm<'static, nrf52840::rtc::Rtc>,
            >,
            bootloader::flash_large_to_small::FlashLargeToSmall<'static, nrf52::nvmc::Nvmc>,
        >,
        bootloader::bootloader::Bootloader::new(
            recv_auto_uart,
            flash_adapter,
            &bootloader_exit,
            pagebuffer,
            &mut bootloader::bootloader::BUF
        )
    );

    hil::uart::Transmit::set_transmit_client(&base_peripherals.uarte0, bootloader);
    hil::uart::Receive::set_receive_client(&base_peripherals.uarte0, recv_auto_uart);
    hil::uart::Receive::set_receive_client(recv_auto_uart, bootloader);
    hil::flash::HasClient::set_client(flash_adapter, bootloader);

    //--------------------------------------------------------------------------
    // FINAL SETUP AND BOARD BOOT
    //--------------------------------------------------------------------------

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    base_peripherals.clock.low_stop();
    base_peripherals.clock.high_stop();
    base_peripherals.clock.low_start();
    base_peripherals.clock.high_start();
    while !base_peripherals.clock.low_started() {}
    while !base_peripherals.clock.high_started() {}

    let platform = Platform {
        bootloader,
        scheduler: null_scheduler,
    };

    let chip = static_init!(
        nrf52840::chip::NRF52<Nrf52840DefaultPeripherals>,
        nrf52840::chip::NRF52::new(nrf52840_peripherals)
    );
    CHIP = Some(chip);

    // Actually run the bootloader.
    platform.bootloader.start();

    //--------------------------------------------------------------------------
    // MAIN LOOP
    //--------------------------------------------------------------------------

    board_kernel.kernel_loop(
        &platform,
        chip,
        None::<&kernel::ipc::IPC<0>>,
        &main_loop_capability,
    );
}

#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    loop {}
}
