extern crate bootloader_attributes;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    let mut f = bootloader_attributes::get_file();
    bootloader_attributes::write_flags(&mut f, 512, "1.0.1");
    bootloader_attributes::write_attribute(&mut f, "board", "nrf52dk");
    bootloader_attributes::write_attribute(&mut f, "arch", "cortex-m4");
    bootloader_attributes::write_attribute(&mut f, "jldevice", "nrf52");
    bootloader_attributes::write_attribute(&mut f, "appaddr", "0x50000");
}
