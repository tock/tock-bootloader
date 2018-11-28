//! Tock bootloader for the nRF52840dk.

#![no_std]
#![no_main]
#![feature(panic_implementation)]

extern crate capsules;
#[allow(unused_imports)]
#[macro_use(create_capability, debug, debug_verbose, debug_gpio, static_init)]
extern crate kernel;
extern crate bootloader;
extern crate cortexm4;
extern crate nrf52;
extern crate nrf5x;

use core::panic::PanicInfo;

use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::hil;

// // The nRF52840DK LEDs (see back of board)
// const LED1_PIN: usize = 13;
// const LED2_PIN: usize = 14;
// const LED3_PIN: usize = 15;
// const LED4_PIN: usize = 16;

// The nRF52840DK buttons (see back of board)
const BUTTON1_PIN: usize = 11;
// const BUTTON2_PIN: usize = 12;
// const BUTTON3_PIN: usize = 24;
// const BUTTON4_PIN: usize = 25;
const BUTTON_RST_PIN: usize = 18;

const UART_RTS: usize = 5;
const UART_TXD: usize = 6;
const UART_CTS: usize = 7;
const UART_RXD: usize = 8;

include!(concat!(env!("OUT_DIR"), "/attributes.rs"));

static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; 0] = [];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Nrf52Bootloader {
    bootloader: &'static bootloader::bootloader::Bootloader<
        'static,
        bootloader::uart_receive_timeout::UartReceiveTimeout<
            'static,
            VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
        >,
        nrf52::nvmc::Nvmc,
        nrf5x::gpio::GPIOPin,
    >,
}

impl kernel::Platform for Nrf52Bootloader {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        match driver_num {
            _ => f(None),
        }
    }
}

/// Entry point in the vector table called on hard reset.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Loads relocations and clears BSS
    nrf52::init();

    // Make non-volatile memory writable and activate the reset button
    let uicr = nrf52::uicr::Uicr::new();
    nrf52::nvmc::NVMC.erase_uicr();
    nrf52::nvmc::NVMC.configure_writeable();
    while !nrf52::nvmc::NVMC.is_ready() {}
    uicr.set_psel0_reset_pin(BUTTON_RST_PIN);
    while !nrf52::nvmc::NVMC.is_ready() {}
    uicr.set_psel1_reset_pin(BUTTON_RST_PIN);

    // Create capabilities that the board needs to call certain protected kernel
    // functions.
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    // kernel::debug::assign_gpios(
    //     Some(&nrf5x::gpio::PORT[LED1_PIN]),
    //     None,
    //     None,
    // );

    // let led_pins = static_init!(
    //     [(&'static nrf5x::gpio::GPIOPin, capsules::led::ActivationMode); 1],
    //     [
    //         (
    //             &nrf5x::gpio::PORT[LED1_PIN],
    //             capsules::led::ActivationMode::ActiveLow
    //         ),
    //     ]
    // );

    // // LEDs
    // let led = static_init!(
    //     capsules::led::LED<'static, nrf5x::gpio::GPIOPin>,
    //     capsules::led::LED::new(led_pins)
    // );

    // Create main kernel object. This contains the main loop function.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Setup the timer infrastructure for faking uart receive with a timeout.
    let rtc = &nrf5x::rtc::RTC;
    rtc.start();
    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, nrf5x::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&nrf5x::rtc::RTC)
    );
    rtc.set_client(mux_alarm);

    // Setup receive with timeout.
    let recv_auto_virtual_alarm = static_init!(
        VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
        VirtualMuxAlarm::new(mux_alarm)
    );

    let recv_auto_uart = static_init!(
        bootloader::uart_receive_timeout::UartReceiveTimeout<
            'static,
            VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
        >,
        bootloader::uart_receive_timeout::UartReceiveTimeout::new(
            &nrf52::uart::UARTE0,
            recv_auto_virtual_alarm,
            &nrf5x::gpio::PORT[UART_RXD]
        )
    );
    recv_auto_virtual_alarm.set_client(recv_auto_uart);
    nrf5x::gpio::PORT[UART_RXD].set_client(recv_auto_uart);
    recv_auto_uart.initialize();

    // Setup the UART pins
    nrf52::uart::UARTE0.initialize(
        nrf5x::pinmux::Pinmux::new(UART_TXD as u32),
        nrf5x::pinmux::Pinmux::new(UART_RXD as u32),
        nrf5x::pinmux::Pinmux::new(UART_CTS as u32),
        nrf5x::pinmux::Pinmux::new(UART_RTS as u32),
    );

    // Create the bootloader object.
    static mut PAGEBUFFER: nrf52::nvmc::NrfPage = nrf52::nvmc::NrfPage::new();
    let bootloader = static_init!(
        bootloader::bootloader::Bootloader<
            'static,
            bootloader::uart_receive_timeout::UartReceiveTimeout<
                'static,
                VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
            >,
            nrf52::nvmc::Nvmc,
            nrf5x::gpio::GPIOPin,
        >,
        bootloader::bootloader::Bootloader::new(
            recv_auto_uart,
            &mut nrf52::nvmc::NVMC,
            &nrf5x::gpio::PORT[BUTTON1_PIN],
            &mut PAGEBUFFER,
            &mut bootloader::bootloader::BUF
        )
    );
    hil::uart::UART::set_client(&nrf52::uart::UARTE0, bootloader);
    hil::flash::HasClient::set_client(&nrf52::nvmc::NVMC, bootloader);

    // Start all of the clocks. Low power operation will require a better
    // approach than this.
    nrf52::clock::CLOCK.low_stop();
    nrf52::clock::CLOCK.high_stop();

    nrf52::clock::CLOCK.low_set_source(nrf52::clock::LowClockSource::XTAL);
    nrf52::clock::CLOCK.low_start();
    nrf52::clock::CLOCK.high_set_source(nrf52::clock::HighClockSource::XTAL);
    nrf52::clock::CLOCK.high_start();
    while !nrf52::clock::CLOCK.low_started() {}
    while !nrf52::clock::CLOCK.high_started() {}

    let platform = Nrf52Bootloader {
        bootloader: bootloader,
    };

    let chip = static_init!(nrf52::chip::NRF52, nrf52::chip::NRF52::new());

    platform.bootloader.initialize();

    board_kernel.kernel_loop(&platform, chip, None, &main_loop_capability);
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_implementation]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    loop {}
}
