#![feature(const_fn, asm)]
// #![forbid(unsafe_code)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug, debug_gpio)]
extern crate kernel;

pub mod bootloader_entry_gpregret;
