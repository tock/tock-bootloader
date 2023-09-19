extern crate bootloader_attributes;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    let mut f = bootloader_attributes::get_file();
    bootloader_attributes::write_flags(&mut f, "1.1.3", 0x36000);
    bootloader_attributes::write_attribute(&mut f, "board", "clue_nrf52840");
    bootloader_attributes::write_attribute(&mut f, "arch", "cortex-m4");
    bootloader_attributes::write_attribute(&mut f, "appaddr", "0x80000");
}
