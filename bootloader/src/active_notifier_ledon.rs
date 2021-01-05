//! Turn on an LED when entering the bootloader.

use crate::interfaces;

pub struct ActiveNotifierLedon<'a> {
    led: &'a mut dyn kernel::hil::led::Led,
}

impl<'a> ActiveNotifierLedon<'a> {
    pub fn new(led: &'a mut dyn kernel::hil::led::Led) -> ActiveNotifierLedon<'a> {
        led.init();
        ActiveNotifierLedon { led }
    }
}

impl<'a> interfaces::ActiveNotifier for ActiveNotifierLedon<'a> {
    fn active(&mut self) {
        self.led.on();
    }
}
