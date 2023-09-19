// #![forbid(unsafe_code)]
#![no_std]

pub mod active_notifier_ledon;
pub mod active_notifier_null;
pub mod bootloader;
pub mod bootloader_crc;
pub mod bootloader_entry_always;
pub mod bootloader_entry_gpio;
pub mod flash_large_to_small;
pub mod interfaces;
pub mod null_scheduler;
pub mod uart_receive_multiple_timeout;
pub mod uart_receive_timeout;
