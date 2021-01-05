#![feature(const_fn, asm, assoc_char_funcs)]
#![feature(const_raw_ptr_to_usize_cast)]
// #![forbid(unsafe_code)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio)]
extern crate kernel;

pub mod active_notifier_null;
pub mod bootloader;
pub mod bootloader_crc;
pub mod bootloader_entry_always;
pub mod bootloader_entry_gpio;
pub mod flash_large_to_small;
pub mod interfaces;
pub mod uart_receive_multiple_timeout;
pub mod uart_receive_timeout;
