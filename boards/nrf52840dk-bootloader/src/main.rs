//! Tock kernel for the Nordic Semiconductor nRF52840 development kit (DK).
//!
//! It is based on nRF52840 SoC (Cortex M4 core with a BLE transceiver) with
//! many exported I/O and peripherals.

#![no_std]
#![no_main]
#![feature(panic_implementation)]
#![deny(missing_docs)]

extern crate capsules;
#[allow(unused_imports)]
#[macro_use(create_capability, debug, debug_verbose, debug_gpio, static_init)]
extern crate kernel;
extern crate cortexm4;
extern crate nrf52;
extern crate nrf5x;
extern crate bootloader;

use core::panic::PanicInfo;

use capsules::virtual_alarm::VirtualMuxAlarm;
// use capsules::virtual_spi::MuxSpiMaster;
// use capsules::virtual_uart::{UartDevice, UartMux};
use kernel::capabilities;
use kernel::hil;
// use kernel::hil::entropy::Entropy32;
// use kernel::hil::rng::Rng;
use nrf5x::rtc::Rtc;

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

// const SPI_MOSI: usize = 20;
// const SPI_MISO: usize = 21;
// const SPI_CLK: usize = 19;

static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; 0] =
    [];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

/// Supported drivers by the platform
pub struct Nrf52Bootloader {
    bootloader: &'static bootloader::bootloader::Bootloader<
        'static,
        bootloader::uart_receive_timeout::UartReceiveTimeout,
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
    // let process_management_capability =
    //     create_capability!(capabilities::ProcessManagementCapability);
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    // let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    let rtc = &nrf5x::rtc::RTC;
    rtc.start();
    let mux_alarm = static_init!(
        capsules::virtual_alarm::MuxAlarm<'static, nrf5x::rtc::Rtc>,
        capsules::virtual_alarm::MuxAlarm::new(&nrf5x::rtc::RTC)
    );
    rtc.set_client(mux_alarm);



    let console_recv_auto_uart = static_init!(
        bootloader::uart_receive_timeout::UartReceiveTimeout,
        bootloader::uart_receive_timeout::UartReceiveTimeout::new(&nrf52::uart::UARTE0)
        );
    hil::uart::UART::set_client(&nrf52::uart::UARTE0, console_recv_auto_uart);

    // Setup the UART pins
    nrf52::uart::UARTE0.initialize(
        nrf5x::pinmux::Pinmux::new(UART_TXD as u32),
        nrf5x::pinmux::Pinmux::new(UART_RXD as u32),
        nrf5x::pinmux::Pinmux::new(UART_CTS as u32),
        nrf5x::pinmux::Pinmux::new(UART_RTS as u32),
    );

    // // Create a shared UART channel for the console and for kernel debug.
    // let uart_mux = static_init!(
    //     UartMux<'static>,
    //     UartMux::new(
    //         console_recv_auto_uart,
    //         &mut capsules::virtual_uart::RX_BUF,
    //         115200
    //     )
    // );
    // hil::uart::UART::set_client(console_recv_auto_uart, uart_mux);


    // // Create virtual device for kernel debug.
    // let debugger_uart = static_init!(UartDevice, UartDevice::new(uart_mux, false));
    // debugger_uart.setup();
    // let debugger = static_init!(
    //     kernel::debug::DebugWriter,
    //     kernel::debug::DebugWriter::new(
    //         debugger_uart,
    //         &mut kernel::debug::OUTPUT_BUF,
    //         &mut kernel::debug::INTERNAL_BUF,
    //     )
    // );
    // hil::uart::UART::set_client(debugger_uart, debugger);

    // let debug_wrapper = static_init!(
    //     kernel::debug::DebugWriterWrapper,
    //     kernel::debug::DebugWriterWrapper::new(debugger)
    // );
    // kernel::debug::set_debug_writer_wrapper(debug_wrapper);





    static mut PAGEBUFFER: nrf52::nvmc::NrfPage = nrf52::nvmc::NrfPage::new();

    // // Create a UartDevice for the bootloader.
    // // This is probably temporary until we no longer need to share this with
    // // the debug console.
    // let bootloader_uart = static_init!(UartDevice, UartDevice::new(uart_mux, true));
    // bootloader_uart.setup();


    // let console = static_init!(
    //     capsules::console::Console<UartDevice>,
    //     capsules::console::Console::new(
    //         console_uart,
    //         115200,
    //         &mut capsules::console::WRITE_BUF,
    //         &mut capsules::console::READ_BUF,
    //         board_kernel.create_grant(&memory_allocation_capability)
    //     )
    // );
    // kernel::hil::uart::UART::set_client(console_uart, console);
    // console.initialize();

    // Create the bootloader object.
    let bootloader = static_init!(
        bootloader::bootloader::Bootloader<
            'static,
            bootloader::uart_receive_timeout::UartReceiveTimeout,
            nrf52::nvmc::Nvmc,
            nrf5x::gpio::GPIOPin,
        >,
        bootloader::bootloader::Bootloader::new(
            console_recv_auto_uart,
            &mut nrf52::nvmc::NVMC,
            &nrf5x::gpio::PORT[BUTTON1_PIN],
            &mut PAGEBUFFER,
            &mut bootloader::bootloader::BUF
        )
    );
    hil::uart::UART::set_client(console_recv_auto_uart, bootloader);
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

    // debug!("Initialization complete. Entering main loop\r");



    board_kernel.kernel_loop(&platform, chip, None, &main_loop_capability);
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_implementation]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    loop {}
}

