#![feature(const_fn, const_cell_new)]
#![forbid(unsafe_code)]
#![no_std]

#[allow(unused_imports)]
#[macro_use(debug)]
extern crate kernel;

pub mod bootloader;
pub mod bootloader_crc;
