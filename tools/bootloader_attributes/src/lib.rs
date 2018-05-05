use std::env;
use std::fs::File;
use std::io::Write;
use std::iter;
use std::path::Path;

// This is the name of the file that will get generated with the static
// attributes in them.
pub static ATTRIBUTES_FILE: &'static str = "attributes.rs";

/// Write the "flags" region as an array.
pub fn write_flags<W: Write>(dest: &mut W, page_size: usize, version: &str) {
    let _ = write!(
        dest,
        "
#[link_section=\".flags\"]
#[no_mangle]
pub static FLAGS: [u8; {}] = [
    ",
        page_size
    );

    // Boot in bootloader identifier flag.
    for byte in "TOCKBOOTLOADER".bytes().chain(iter::repeat(0)).take(14) {
        let _ = write!(dest, "{:#x}, ", byte);
    }

    // Write up to 8 bytes for the bootloader versrion.
    for byte in version.bytes().chain(iter::repeat(0)).take(8) {
        let _ = write!(dest, "{:#x}, ", byte);
    }

    // Fill in the rest of the 0 bytes
    for _ in 0..490 {
        let _ = write!(dest, "{:#x}, ", 0);
    }

    // And finish the array
    let _ = write!(dest, " ]; ");
}

/// Takes an attribute name and value and writes valid Rust to create a
/// bootloader attribute.
pub fn write_attribute<W: Write>(dest: &mut W, name: &str, value: &str) {
    let _ = write!(
        dest,
        "
#[link_section=\".attribute.{}\"]
#[no_mangle]
pub static ATTRIBUTE_{}: [u8; 64] = [
    ",
        name,
        name.to_ascii_uppercase()
    );

    // Write up to 8 bytes of name ; zero-pad up to 8 bytes
    for byte in name.bytes().chain(iter::repeat(0)).take(8) {
        let _ = write!(dest, "{:#x}, ", byte);
    }

    // attribute length
    let _ = write!(dest, "{:#x}, ", value.len());

    // Write up to 55 bytes of value ; zero-pad up to 55 bytes
    for byte in value.bytes().chain(iter::repeat(0)).take(55) {
        let _ = write!(dest, "{:#x}, ", byte);
    }

    // And finish the array
    let _ = write!(dest, " ]; ");
}

pub fn get_file() -> File {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(ATTRIBUTES_FILE);
    let f = File::create(&dest_path).unwrap();
    f
}
