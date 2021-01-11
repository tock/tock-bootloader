//! Decide to enter bootloader unconditionally.

use crate::interfaces;

pub struct BootloaderEntryAlways {}

impl BootloaderEntryAlways {
    pub fn new() -> BootloaderEntryAlways {
        BootloaderEntryAlways {}
    }
}

impl interfaces::BootloaderEntry for BootloaderEntryAlways {
    fn stay_in_bootloader(&self) -> bool {
        true
    }
}
