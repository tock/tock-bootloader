//! Board file for Imix bootloader.

#![no_std]
#![no_main]
#![feature(asm, const_fn, lang_items, panic_implementation)]

extern crate bootloader;
extern crate cortexm4;
#[macro_use(create_capability, static_init)]
extern crate kernel;
extern crate capsules;
extern crate sam4l;

use core::panic::PanicInfo;

use kernel::capabilities;
use kernel::hil;
use kernel::hil::Controller;
use kernel::Platform;

include!(concat!(env!("OUT_DIR"), "/attributes.rs"));

// No processes are supported.
static mut PROCESSES: [Option<&'static kernel::procs::ProcessType>; 0] = [];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x2000] = [0; 0x2000];

struct ImixBootloader {
    bootloader: &'static bootloader::bootloader::Bootloader<
        'static,
        sam4l::usart::USART,
        sam4l::flashcalw::FLASHCALW,
        sam4l::gpio::GPIOPin,
    >,
}

impl Platform for ImixBootloader {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        // Bootloader does not support apps.
        match driver_num {
            _ => f(None),
        }
    }
}

unsafe fn set_pin_primary_functions() {
    use sam4l::gpio::PeripheralFunction::{A, B, C, E};
    use sam4l::gpio::{PA, PB, PC};

    // Right column: Imix pin name
    // Left  column: SAM4L peripheral function
    PA[04].configure(Some(A)); // AD0         --  ADCIFE AD0
    PA[05].configure(Some(A)); // AD1         --  ADCIFE AD1
    PA[06].configure(Some(C)); // EXTINT1     --  EIC EXTINT1
    PA[07].configure(Some(A)); // AD1         --  ADCIFE AD2
    PA[08].configure(None); //... RF233 IRQ   --  GPIO pin
    PA[09].configure(None); //... RF233 RST   --  GPIO pin
    PA[10].configure(None); //... RF233 SLP   --  GPIO pin
    PA[13].configure(None); //... TRNG EN     --  GPIO pin
    PA[14].configure(None); //... TRNG_OUT    --  GPIO pin
    PA[17].configure(None); //... NRF INT     -- GPIO pin
    PA[18].configure(Some(A)); // NRF CLK     -- USART2_CLK
    PA[20].configure(None); //... D8          -- GPIO pin
    PA[21].configure(Some(E)); // TWI2 SDA    -- TWIM2_SDA
    PA[22].configure(Some(E)); // TWI2 SCL    --  TWIM2 TWCK
    PA[25].configure(Some(A)); // USB_N       --  USB DM
    PA[26].configure(Some(A)); // USB_P       --  USB DP
    PB[00].configure(Some(A)); // TWI1_SDA    --  TWIMS1 TWD
    PB[01].configure(Some(A)); // TWI1_SCL    --  TWIMS1 TWCK
    PB[02].configure(Some(A)); // AD3         --  ADCIFE AD3
    PB[03].configure(Some(A)); // AD4         --  ADCIFE AD4
    PB[04].configure(Some(A)); // AD5         --  ADCIFE AD5
    PB[05].configure(Some(A)); // VHIGHSAMPLE --  ADCIFE AD6
    PB[06].configure(None); //... RTS3        --  USART3 RTS !! FTDI DTR BOOTLOADER SELECT
    PB[07].configure(None); //... NRF RESET   --  GPIO
    PB[09].configure(Some(A)); // RX3         --  USART3 RX
    PB[10].configure(Some(A)); // TX3         --  USART3 TX
    PB[11].configure(Some(A)); // CTS0        --  USART0 CTS
    PB[12].configure(Some(A)); // RTS0        --  USART0 RTS
    PB[13].configure(Some(A)); // CLK0        --  USART0 CLK
    PB[14].configure(Some(A)); // RX0         --  USART0 RX
    PB[15].configure(Some(A)); // TX0         --  USART0 TX
    PC[00].configure(Some(A)); // CS2         --  SPI NPCS2
    PC[01].configure(Some(A)); // CS3 (RF233) --  SPI NPCS3
    PC[02].configure(Some(A)); // CS1         --  SPI NPCS1
    PC[03].configure(Some(A)); // CS0         --  SPI NPCS0
    PC[04].configure(Some(A)); // MISO        --  SPI MISO
    PC[05].configure(Some(A)); // MOSI        --  SPI MOSI
    PC[06].configure(Some(A)); // SCK         --  SPI CLK
    PC[07].configure(Some(B)); // RTS2 (BLE)  -- USART2_RTS
    PC[08].configure(Some(E)); // CTS2 (BLE)  -- USART2_CTS
                               //PC[09].configure(None); //... NRF GPIO    -- GPIO
                               //PC[10].configure(None); //... USER LED    -- GPIO
    PC[09].configure(Some(E)); // ACAN1       -- ACIFC comparator
    PC[10].configure(Some(E)); // ACAP1       -- ACIFC comparator
    PC[11].configure(Some(B)); // RX2 (BLE)   -- USART2_RX
    PC[12].configure(Some(B)); // TX2 (BLE)   -- USART2_TX
                               //PC[13].configure(None); //... ACC_INT1    -- GPIO
                               //PC[14].configure(None); //... ACC_INT2    -- GPIO
    PC[13].configure(Some(E)); //... ACBN1    -- ACIFC comparator
    PC[14].configure(Some(E)); //... ACBP1    -- ACIFC comparator
    PC[16].configure(None); //... SENSE_PWR   --  GPIO pin
    PC[17].configure(None); //... NRF_PWR     --  GPIO pin
    PC[18].configure(None); //... RF233_PWR   --  GPIO pin
    PC[19].configure(None); //... TRNG_PWR    -- GPIO Pin
    PC[22].configure(None); //... KERNEL LED  -- GPIO Pin
    PC[24].configure(None); //... USER_BTN    -- GPIO Pin
    PC[25].configure(Some(B)); // LI_INT      --  EIC EXTINT2
    PC[26].configure(None); //... D7          -- GPIO Pin
    PC[27].configure(None); //... D6          -- GPIO Pin
    PC[28].configure(None); //... D5          -- GPIO Pin
    PC[29].configure(None); //... D4          -- GPIO Pin
    PC[30].configure(None); //... D3          -- GPIO Pin
    PC[31].configure(None); //... D2          -- GPIO Pin
}

#[no_mangle]
pub unsafe fn reset_handler() {
    sam4l::init();

    sam4l::pm::PM.setup_system_clock(sam4l::pm::SystemClockSource::PllExternalOscillatorAt48MHz {
        frequency: sam4l::pm::OscillatorFrequency::Frequency16MHz,
        startup_mode: sam4l::pm::OscillatorStartup::FastStart,
    });

    // Source 32Khz and 1Khz clocks from RC23K (SAM4L Datasheet 11.6.8)
    sam4l::bpm::set_ck32source(sam4l::bpm::CK32Source::RC32K);

    set_pin_primary_functions();

    // Create main kernel object. This contains the main loop function.
    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Initialize USART3 for Uart
    sam4l::usart::USART3.set_mode(sam4l::usart::UsartMode::Uart);

    pub static mut PAGEBUFFER: sam4l::flashcalw::Sam4lPage = sam4l::flashcalw::Sam4lPage::new();

    sam4l::flashcalw::FLASH_CONTROLLER.configure();
    let bootloader = static_init!(
        bootloader::bootloader::Bootloader<
            'static,
            sam4l::usart::USART,
            sam4l::flashcalw::FLASHCALW,
            sam4l::gpio::GPIOPin,
        >,
        bootloader::bootloader::Bootloader::new(
            &sam4l::usart::USART3,
            &mut sam4l::flashcalw::FLASH_CONTROLLER,
            &sam4l::gpio::PB[06],
            &mut PAGEBUFFER,
            &mut bootloader::bootloader::BUF
        )
    );
    hil::uart::UART::set_client(&sam4l::usart::USART3, bootloader);
    hil::flash::HasClient::set_client(&sam4l::flashcalw::FLASH_CONTROLLER, bootloader);

    let imix = ImixBootloader {
        bootloader: bootloader,
    };

    let chip = static_init!(sam4l::chip::Sam4l, sam4l::chip::Sam4l::new());

    imix.bootloader.initialize();

    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
    board_kernel.kernel_loop(&imix, chip, None, &main_loop_capability);
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_implementation]
pub unsafe extern "C" fn panic_fmt(_pi: &PanicInfo) -> ! {
    loop {}
}
