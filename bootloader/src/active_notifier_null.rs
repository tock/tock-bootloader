//! Do nothing when entering the bootloader.
//!
//! This means nothing will notifying the user the bootloader has started.

use crate::interfaces;

pub struct ActiveNotifierNull {}

impl ActiveNotifierNull {
    pub fn new() -> ActiveNotifierNull {
        ActiveNotifierNull {}
    }
}

impl interfaces::ActiveNotifier for ActiveNotifierNull {
    fn active(&self) {}
}
