//! Implements the Tock bootloader.

use core::cell::Cell;
use core::cmp;
use kernel::common::take_cell::TakeCell;
use kernel::hil;

use bootloader_crc;

extern crate tockloader_proto;

// Main buffer that commands are received into and sent from.
pub static mut BUF: [u8; 600] = [0; 600];

// Bootloader constants
const ESCAPE_CHAR: u8 = 0xFC;

const RES_PONG: u8 = 0x11;
const RES_INTERNAL_ERROR: u8 = 0x13;
const RES_BADARGS: u8 = 0x14;
const RES_OK: u8 = 0x15;
const RES_UNKNOWN: u8 = 0x16;
const RES_READ_RANGE: u8 = 0x20;
const RES_GET_ATTR: u8 = 0x22;
const RES_CRCIF: u8 = 0x23;
const RES_INFO: u8 = 0x25;

#[derive(Copy, Clone, PartialEq)]
enum State {
    Idle,
    Info,
    ErasePage,
    GetAttribute {
        index: u8,
    },
    SetAttribute {
        index: u8,
    },
    WriteFlashPage,
    ReadRange {
        address: u32,
        length: u16,
        remaining_length: u16,
    },
    Crc {
        address: u32,
        remaining_length: u32,
        crc: u32,
    },
}

pub struct Bootloader<
    'a,
    U: hil::uart::UARTAdvanced + 'a,
    F: hil::flash::Flash + 'static,
    G: hil::gpio::Pin + 'a,
> {
    uart: &'a U,
    flash: &'a F,
    select_pin: &'a G,
    page_buffer: TakeCell<'static, F::Page>,
    buffer: TakeCell<'static, [u8]>,
    state: Cell<State>,
}

impl<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'a, G: hil::gpio::Pin + 'a>
    Bootloader<'a, U, F, G>
{
    pub fn new(
        uart: &'a U,
        flash: &'a F,
        select_pin: &'a G,
        page_buffer: &'static mut F::Page,
        buffer: &'static mut [u8],
    ) -> Bootloader<'a, U, F, G> {
        Bootloader {
            uart: uart,
            flash: flash,
            select_pin: select_pin,
            page_buffer: TakeCell::new(page_buffer),
            buffer: TakeCell::new(buffer),
            state: Cell::new(State::Idle),
        }
    }

    pub fn initialize(&self) {
        // Setup UART and start listening.
        self.uart.init(hil::uart::UARTParams {
            baud_rate: 115200,
            stop_bits: hil::uart::StopBits::One,
            parity: hil::uart::Parity::None,
            hw_flow_control: false,
        });

        self.select_pin.make_input();

        // Check the select pin to see if we should enter bootloader mode.
        let mut samples = 10000;
        let mut active = 0;
        let mut inactive = 0;
        while samples > 0 {
            if self.select_pin.read() == false {
                active += 1;
            } else {
                inactive += 1;
            }
            samples -= 1;
        }

        if active > inactive || true {
            // Looks like we do want bootloader mode.

            self.buffer.take().map(|buffer| {
                self.uart.receive_automatic(buffer, 250);
            });
        } else {
            // Jump to the kernel and start the real code.
            self.jump();
        }
    }

    // Helper function for sending single byte responses.
    fn send_response(&self, response: u8) {
        self.buffer.take().map(|buffer| {
            buffer[0] = ESCAPE_CHAR;
            buffer[1] = response;
            self.uart.transmit(buffer, 2);
        });
    }

    fn jump(&self) {

        // asm!(
        //         ".syntax unified                        \n\
        //         .section .text.jumpfunc                 \n\
        //         .global jump_into_user_code             \n\
        //         .thumb_func                             \n\
        //     jump_into_user_code:                        \n\
        //         ldr r0, =0x10000    // The address of the payload's .vectors                                       \n\
        //         ldr r1, =0xe000ed08 // The address of the VTOR register (0xE000E000(SCS) + 0xD00(SCB) + 0x8(VTOR)) \n\
        //         str r0, [r1]        // Move the payload's VT address into the VTOR register                        \n\
        //         ldr r1, [r0]        // Move the payload's initial SP into r1                                       \n\
        //         mov sp, r1          // Set our SP to that                                                          \n\
        //         ldr r0, [r0, #4]    // Load the payload's ENTRY into r0                                            \n\
        //         bx  r0              // Whoopee"
        //     );
    }
}

impl<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'a, G: hil::gpio::Pin + 'a>
    hil::uart::Client for Bootloader<'a, U, F, G>
{
    fn transmit_complete(&self, buffer: &'static mut [u8], error: hil::uart::Error) {
        if error != hil::uart::Error::CommandComplete {
            // self.led.clear();
        } else {
            match self.state.get() {
                // Check if there is more to be read, and if so, read it and
                // send it.
                State::ReadRange {
                    address,
                    length: _,
                    remaining_length,
                } => {
                    // We have sent some of the read range to the client.
                    // We are either done, or need to setup the next read.
                    if remaining_length == 0 {
                        self.state.set(State::Idle);
                        self.uart.receive_automatic(buffer, 250);
                    } else {
                        self.buffer.replace(buffer);
                        self.page_buffer.take().map(move |page| {
                            let page_size = page.as_mut().len();
                            self.flash.read_page(address as usize / page_size, page);
                        });
                    }
                }

                _ => {
                    self.uart.receive_automatic(buffer, 250);
                }
            }
        }
    }

    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, error: hil::uart::Error) {
        if error != hil::uart::Error::CommandComplete {
            // self.led.clear();
            return;
        }

        // Tool to parse incoming bootloader messages.
        let mut decoder = tockloader_proto::CommandDecoder::new();
        // Whether we want to reset the position in the buffer in the
        // decoder.
        let mut need_reset = false;

        // Loop through the buffer and pass it to the decoder.
        for i in 0..rx_len {
            match decoder.receive(buffer[i]) {
                Ok(None) => {}
                Ok(Some(tockloader_proto::Command::Ping)) => {
                    self.buffer.replace(buffer);
                    self.send_response(RES_PONG);
                    break;
                }
                Ok(Some(tockloader_proto::Command::Reset)) => {
                    need_reset = true;
                    self.buffer.replace(buffer);
                    break;
                }
                Ok(Some(tockloader_proto::Command::Info)) => {
                    self.state.set(State::Info);
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        self.flash.read_page(2, page);
                    });
                    break;
                }
                Ok(Some(tockloader_proto::Command::ReadRange { address, length })) => {
                    self.state.set(State::ReadRange {
                        address,
                        length,
                        remaining_length: length,
                    });
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        let page_size = page.as_mut().len();
                        self.flash.read_page(address as usize / page_size, page);
                    });
                    break;
                }
                Ok(Some(tockloader_proto::Command::WritePage { address, data })) => {
                    self.page_buffer.take().map(move |page| {
                        let page_size = page.as_mut().len();
                        if page_size != data.len() {
                            // Error if we didn't get exactly a page of data
                            // to write to flash.
                            buffer[0] = ESCAPE_CHAR;
                            buffer[1] = RES_BADARGS;
                            self.page_buffer.replace(page);
                            self.state.set(State::Idle);
                            self.uart.transmit(buffer, 2);
                        } else {
                            // Otherwise copy into page buffer and write to
                            // flash.
                            for i in 0..page_size {
                                page.as_mut()[i] = data[i];
                            }
                            self.state.set(State::WriteFlashPage);
                            self.buffer.replace(buffer);
                            self.flash.write_page(address as usize / page_size, page);
                        }
                    });
                    break;
                }
                Ok(Some(tockloader_proto::Command::ErasePage { address })) => {
                    self.state.set(State::ErasePage);
                    self.buffer.replace(buffer);
                    let page_size = self.page_buffer.map_or(512, |page| page.as_mut().len());
                    self.flash.erase_page(address as usize / page_size);
                    break;
                }
                Ok(Some(tockloader_proto::Command::CrcIntFlash { address, length })) => {
                    self.state.set(State::Crc {
                        address,
                        remaining_length: length,
                        crc: 0xFFFFFFFF,
                    });
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        let page_size = page.as_mut().len();
                        self.flash.read_page(address as usize / page_size, page);
                    });
                    break;
                }
                Ok(Some(tockloader_proto::Command::GetAttr { index })) => {
                    self.state.set(State::GetAttribute { index: index });
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        self.flash.read_page(3 + (index as usize / 8), page);
                    });
                    break;
                }
                Ok(Some(tockloader_proto::Command::SetAttr { index, key, value })) => {
                    self.state.set(State::SetAttribute { index });

                    // Copy the key and value into the buffer so it can be added
                    // to the page buffer when needed.
                    for i in 0..8 {
                        buffer[i] = key[i];
                    }
                    buffer[8] = value.len() as u8;
                    for i in 0..55 {
                        // Copy in the value, otherwise clear to zero.
                        if i < value.len() {
                            buffer[9 + i] = value[i];
                        } else {
                            buffer[9 + i] = 0;
                        }
                    }
                    self.buffer.replace(buffer);

                    // Initiate things by reading the correct flash page that
                    // needs to be updated.
                    self.page_buffer.take().map(move |page| {
                        self.flash.read_page(3 + (index as usize / 8), page);
                    });
                    break;
                }
                Ok(Some(_)) => {
                    self.send_response(RES_UNKNOWN);
                    break;
                }
                Err(_) => {
                    self.send_response(RES_INTERNAL_ERROR);
                    break;
                }
            };
        }

        // Artifact of the original implementation of the bootloader protocol
        // is the need to reset the pointer internal to the bootloader receive
        // state machine.
        if need_reset {
            decoder.reset();

            self.buffer.take().map(|buffer| {
                self.uart.receive_automatic(buffer, 250);
            });
        }
    }
}

impl<'a, U: hil::uart::UARTAdvanced + 'a, F: hil::flash::Flash + 'a, G: hil::gpio::Pin + 'a>
    hil::flash::Client<F> for Bootloader<'a, U, F, G>
{
    fn read_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {
        match self.state.get() {
            // We just read the bootloader info page (page 2). Extract the
            // version and generate a response JSON blob.
            State::Info => {
                self.state.set(State::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_INFO;

                    // "{\"version\":\"%s\", \"name\":\"Tock Bootloader\"}"

                    // Version string is at most 8 bytes long, and starts
                    // at index 14 in the bootloader page.
                    for i in 0..8 {
                        let b = pagebuffer.as_mut()[i + 14];
                        if b == 0 {
                            break;
                        }
                        buffer[i + 2] = b;
                    }
                    for i in 10..195 {
                        buffer[i] = 0;
                    }

                    self.page_buffer.replace(pagebuffer);
                    self.uart.transmit(buffer, 195);
                });
            }

            // We just read the correct page for this attribute. Copy it to
            // the out buffer and send it back to the client.
            State::GetAttribute { index } => {
                self.state.set(State::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_GET_ATTR;
                    let mut j = 2;
                    for i in 0..64 {
                        let b = pagebuffer.as_mut()[(((index as usize) % 8) * 64) + i];
                        if b == ESCAPE_CHAR {
                            // Need to escape the escape character.
                            buffer[j] = ESCAPE_CHAR;
                            j += 1;
                        }
                        buffer[j] = b;
                        j += 1;
                    }

                    self.page_buffer.replace(pagebuffer);
                    self.uart.transmit(buffer, j);
                });
            }

            // We need to update the page we just read with the new attribute,
            // and then write that all back to flash.
            State::SetAttribute { index } => {
                self.buffer.map(move |buffer| {
                    // Copy the first 64 bytes of the buffer into the correct
                    // spot in the page.
                    let start_index = ((index as usize) % 8) * 64;
                    for i in 0..64 {
                        pagebuffer.as_mut()[start_index + i] = buffer[i];
                    }
                    self.flash.write_page(3 + (index as usize / 8), pagebuffer);
                });
            }

            // Pass what we have read so far to the client.
            State::ReadRange {
                address,
                length,
                remaining_length,
            } => {
                // Take what we need to read out of this page and send it
                // on uart. If this is the first message be sure to send the
                // header.
                self.buffer.take().map(move |buffer| {
                    let mut index = 0;
                    if length == remaining_length {
                        buffer[0] = ESCAPE_CHAR;
                        buffer[1] = RES_READ_RANGE;
                        index = 2;
                    }

                    let page_size = pagebuffer.as_mut().len();
                    // This will get us our offset into the page.
                    let page_index = address as usize % page_size;
                    // Length is either the rest of the page or how much we have left.
                    let len = cmp::min(page_size - page_index, remaining_length as usize);
                    // Make sure we don't overflow the buffer.
                    let copy_len = cmp::min(len, buffer.len() - index);

                    // Copy what we read from the page buffer to the user buffer.
                    // Keep track of how much was actually copied.
                    let mut actually_copied = 0;
                    for i in 0..copy_len {
                        // Make sure we don't overflow the buffer. We need to
                        // have at least two open bytes in the buffer
                        if index >= (buffer.len() - 1) {
                            break;
                        }

                        // Normally do the copy and check if this needs to be
                        // escaped.
                        actually_copied += 1;
                        let b = pagebuffer.as_mut()[page_index + i];
                        if b == ESCAPE_CHAR {
                            // Need to escape the escape character.
                            buffer[index] = ESCAPE_CHAR;
                            index += 1;
                        }
                        buffer[index] = b;
                        index += 1;
                    }

                    // Update our state.
                    let new_address = address as usize + actually_copied;
                    let new_remaining_length = remaining_length as usize - actually_copied;
                    self.state.set(State::ReadRange {
                        address: new_address as u32,
                        length,
                        remaining_length: new_remaining_length as u16,
                    });

                    // And send the buffer to the client.
                    self.page_buffer.replace(pagebuffer);
                    self.uart.transmit(buffer, index);
                });
            }

            // We have some data to calculate the CRC on.
            State::Crc {
                address,
                remaining_length,
                crc,
            } => {
                let page_size = pagebuffer.as_mut().len();
                // This will get us our offset into the page.
                let page_index = address as usize % page_size;
                // Length is either the rest of the page or how much we have left.
                let len = cmp::min(page_size - page_index, remaining_length as usize);

                // Iterate all bytes in the page that are relevant to the CRC
                // and include them in the CRC calculation.
                let mut new_crc = crc;
                for i in 0..len {
                    let v1 = (new_crc ^ pagebuffer.as_mut()[page_index + i] as u32) & 0xFF;
                    let v2 = bootloader_crc::CRC32_TABLE[v1 as usize];
                    new_crc = v2 ^ (new_crc >> 8);
                }

                // Update our state.
                let new_address = address + len as u32;
                let new_remaining_length = remaining_length - len as u32;

                // Check if we are done
                if new_remaining_length == 0 {
                    // Last XOR before sending to client.
                    new_crc = new_crc ^ 0xFFFFFFFF;

                    self.state.set(State::Idle);
                    self.buffer.take().map(move |buffer| {
                        buffer[0] = ESCAPE_CHAR;
                        buffer[1] = RES_CRCIF;
                        buffer[2] = ((new_crc >> 0) & 0xFF) as u8;
                        buffer[3] = ((new_crc >> 8) & 0xFF) as u8;
                        buffer[4] = ((new_crc >> 16) & 0xFF) as u8;
                        buffer[5] = ((new_crc >> 24) & 0xFF) as u8;
                        // And send the buffer to the client.
                        self.page_buffer.replace(pagebuffer);
                        self.uart.transmit(buffer, 6);
                    });
                } else {
                    // More CRC to do!
                    self.state.set(State::Crc {
                        address: new_address,
                        remaining_length: new_remaining_length,
                        crc: new_crc,
                    });
                    self.flash
                        .read_page(new_address as usize / page_size, pagebuffer);
                }
            }

            _ => {}
        }
    }

    fn write_complete(&self, pagebuffer: &'static mut F::Page, _error: hil::flash::Error) {
        self.page_buffer.replace(pagebuffer);

        match self.state.get() {
            // Writing flash page done, send OK.
            State::WriteFlashPage => {
                self.state.set(State::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_OK;
                    // buffer[1] = 0x99;
                    self.uart.transmit(buffer, 2);
                });
            }

            // Attribute writing done, send an OK response.
            State::SetAttribute { index: _ } => {
                self.state.set(State::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_OK;
                    self.uart.transmit(buffer, 2);
                });
            }

            _ => {
                self.buffer.take().map(|buffer| {
                    self.uart.receive_automatic(buffer, 250);
                });
            }
        }
    }

    fn erase_complete(&self, _error: hil::flash::Error) {
        match self.state.get() {
            // Page erased, return OK
            State::ErasePage => {
                self.state.set(State::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_OK;
                    self.uart.transmit(buffer, 2);
                });
            }

            _ => {
                self.buffer.take().map(|buffer| {
                    self.uart.receive_automatic(buffer, 250);
                });
            }
        }
    }
}
