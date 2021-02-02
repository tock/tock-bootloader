extern crate bootloader_attributes;

fn main() {
    println!("cargo:rerun-if-changed=layout.ld");
    println!("cargo:rerun-if-changed=../kernel_layout.ld");

    let mut f = bootloader_attributes::get_file();
    bootloader_attributes::write_flags(&mut f, "1.1.1", 0x8000);
    bootloader_attributes::write_attribute(&mut f, "board", "microbit_v2");
    bootloader_attributes::write_attribute(&mut f, "arch", "cortex-m4");
    bootloader_attributes::write_attribute(&mut f, "appaddr", "0x40000");
}
