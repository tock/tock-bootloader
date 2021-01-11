//! Decide to enter bootloader based on GPIO pin.
//!
//! This is often connected to a UART RTS pin so that the host-side UART
//! hardware can toggle the pin automatically to enter bootloader mode.

use kernel::hil;

use crate::interfaces;

pub struct BootloaderEntryGpio<'a, G: hil::gpio::Pin + 'a> {
    select_pin: &'a G,
}

impl<'a, G: hil::gpio::Pin + 'a> BootloaderEntryGpio<'a, G> {
    pub fn new(select_pin: &'a G) -> BootloaderEntryGpio<'a, G> {
        BootloaderEntryGpio { select_pin }
    }
}

impl<'a, G: hil::gpio::Pin + 'a> interfaces::BootloaderEntry for BootloaderEntryGpio<'a, G> {
    fn stay_in_bootloader(&self) -> bool {
        self.select_pin.make_input();

        // Check the select pin to see if we should enter bootloader mode.
        let mut samples = 10000;
        let mut active = 0;
        let mut inactive = 0;
        while samples > 0 {
            if self.select_pin.read() == false {
                active += 1;
            } else {
                inactive += 1;
            }
            samples -= 1;
        }

        active > inactive
    }
}
