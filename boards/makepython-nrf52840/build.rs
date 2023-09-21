extern crate bootloader_attributes;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    let mut f = bootloader_attributes::get_file();

    let version = if let Ok(v) = env::var("BOOTLOADER_VERSION") {
        v
    } else {
        String::from("1.1.3")
    };

    bootloader_attributes::write_flags(&mut f, &version, 0x10000);
    bootloader_attributes::write_attribute(&mut f, "board", "makepython-nrf52840");
    bootloader_attributes::write_attribute(&mut f, "arch", "cortex-m4");
    bootloader_attributes::write_attribute(&mut f, "appaddr", "0x50000");
    if let Ok(bootloader) = env::var("BOOTLOADER_HASH") {
        bootloader_attributes::write_attribute(&mut f, "boothash", &bootloader);
    }
    if let Ok(bootloader_kernel) = env::var("BOOTLOADER_KERNEL_HASH") {
        bootloader_attributes::write_attribute(&mut f, "kernhash", &bootloader_kernel);
    }
}
