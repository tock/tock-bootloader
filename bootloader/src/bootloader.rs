//! Implements the Tock bootloader.

use core::cell::Cell;
use core::cmp;

use kernel::common::cells::TakeCell;
use kernel::common::cells::VolatileCell;
use kernel::common::StaticRef;
use kernel::hil;

use bootloader_crc;
use interfaces;

// Main buffer that commands are received into and sent from.
// Need a buffer big enough for 512 byte pages.
pub static mut BUF: [u8; 600] = [0; 600];

// How long to wait, in bit periods, after receiving a byte for the next
// byte before timing out and calling `receive_complete`.
const UART_RECEIVE_TIMEOUT: u8 = 100;

// Get the addresses in flash of key components from the linker file.
extern "C" {
    static _flags_address: u8;
    static _attributes_address: u8;
    static _stext: u8;
    static _etext: u8;
}

// Bootloader constants
const ESCAPE_CHAR: u8 = 0xFC;

const RES_PONG: u8 = 0x11;
const RES_BADADDR: u8 = 0x12;
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
    SetStartAddress {
        address: u32,
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

/// This struct handles whether we should enter the bootloader or go straight to
/// the kernel.
pub struct BootloaderEnterer<'a> {
    entry_decider: &'a dyn interfaces::BootloaderEntry,
    jumper: &'a dyn interfaces::Jumper,
    active_notifier: &'a mut dyn interfaces::ActiveNotifier,
    /// This is the address of flash where the flags region of the bootloader
    /// start. We need this to determine what address to jump to.
    bootloader_flags_address: u32,
}

impl<'a> BootloaderEnterer<'a> {
    pub fn new(
        entry_decider: &'a dyn interfaces::BootloaderEntry,
        jumper: &'a dyn interfaces::Jumper,
        active_notifier: &'a mut dyn interfaces::ActiveNotifier,
    ) -> BootloaderEnterer<'a> {
        BootloaderEnterer {
            entry_decider,
            jumper,
            active_notifier,
            bootloader_flags_address: unsafe { (&_flags_address as *const u8) as u32 },
        }
    }

    pub fn check(&mut self) {
        if !self.entry_decider.stay_in_bootloader() {
            // Jump to the kernel and start the real code.
            self.jump();
        } else {
            // Staying in the bootloader, allow a custom active notification to
            // start.
            self.active_notifier.active();
        }
    }

    fn jump(&self) {
        // Address of the start address in the flags region is 32 bytes from the start.
        let start_address_memory_location = self.bootloader_flags_address + 32;

        let start_address_ptr: StaticRef<VolatileCell<u32>> =
            unsafe { StaticRef::new(start_address_memory_location as *const VolatileCell<u32>) };

        let start_address = start_address_ptr.get();

        self.jumper.jump(start_address);
    }
}

/// The main bootloader code.
pub struct Bootloader<'a, U: hil::uart::UartAdvanced<'a> + 'a, F: hil::flash::Flash + 'static> {
    uart: &'a U,
    flash: &'a F,
    reset_function: &'a (dyn Fn() + 'a),
    page_buffer: TakeCell<'static, F::Page>,
    buffer: TakeCell<'static, [u8]>,
    state: Cell<State>,
    flags_address: usize,
    attributes_address: usize,
    /// Address of the bootloader in flash.
    bootloader_address: u32,
    /// Address after the bootloader in flash.
    bootloader_end_address: u32,
}

impl<'a, U: hil::uart::UartAdvanced<'a> + 'a, F: hil::flash::Flash + 'a> Bootloader<'a, U, F> {
    pub fn new(
        uart: &'a U,
        flash: &'a F,
        reset_function: &'a (dyn Fn() + 'a),
        page_buffer: &'static mut F::Page,
        buffer: &'static mut [u8],
    ) -> Bootloader<'a, U, F> {
        Bootloader {
            uart: uart,
            flash: flash,
            reset_function: reset_function,
            page_buffer: TakeCell::new(page_buffer),
            buffer: TakeCell::new(buffer),
            state: Cell::new(State::Idle),
            flags_address: unsafe { (&_flags_address as *const u8) as usize },
            attributes_address: unsafe { (&_attributes_address as *const u8) as usize },
            bootloader_address: unsafe { (&_stext as *const u8) as u32 },
            bootloader_end_address: unsafe { (&_etext as *const u8) as u32 },
        }
    }

    pub fn start(&self) {
        // Setup UART and start listening.
        self.uart.configure(hil::uart::Parameters {
            baud_rate: 115200,
            width: hil::uart::Width::Eight,
            stop_bits: hil::uart::StopBits::One,
            parity: hil::uart::Parity::None,
            hw_flow_control: false,
        });

        self.buffer.take().map(|buffer| {
            self.uart
                .receive_automatic(buffer, buffer.len(), UART_RECEIVE_TIMEOUT);
        });
    }

    // Helper function for sending single byte responses.
    fn send_response(&self, response: u8) {
        self.buffer.take().map(|buffer| {
            buffer[0] = ESCAPE_CHAR;
            buffer[1] = response;
            self.uart.transmit_buffer(buffer, 2);
        });
    }
}

impl<'a, U: hil::uart::UartAdvanced<'a> + 'a, F: hil::flash::Flash + 'a> hil::uart::TransmitClient
    for Bootloader<'a, U, F>
{
    fn transmitted_buffer(
        &self,
        buffer: &'static mut [u8],
        _tx_len: usize,
        error: kernel::ReturnCode,
    ) {
        if error != kernel::ReturnCode::SUCCESS {
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
                        self.uart
                            .receive_automatic(buffer, buffer.len(), UART_RECEIVE_TIMEOUT);
                    } else {
                        self.buffer.replace(buffer);
                        self.page_buffer.take().map(move |page| {
                            let page_size = page.as_mut().len();
                            let _ = self.flash.read_page(address as usize / page_size, page);
                        });
                    }
                }

                _ => {
                    self.uart
                        .receive_automatic(buffer, buffer.len(), UART_RECEIVE_TIMEOUT);
                }
            }
        }
    }
}

impl<'a, U: hil::uart::UartAdvanced<'a> + 'a, F: hil::flash::Flash + 'a> hil::uart::ReceiveClient
    for Bootloader<'a, U, F>
{
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rval: kernel::ReturnCode,
        _error: hil::uart::Error,
    ) {
        if rval != kernel::ReturnCode::SUCCESS {
            return;
        }

        // Tool to parse incoming bootloader messages.
        // This is currently allocated on the stack, but it too needs a big
        // buffer, and we need to do something about that.
        let mut decoder = tock_bootloader_protocol::CommandDecoder::new();
        // Whether we want to reset the position in the buffer in the
        // decoder.
        let mut need_reset = false;

        // Loop through the buffer and pass it to the decoder.
        for i in 0..rx_len {
            // Artifact of the original implementation of the bootloader
            // protocol is the need to reset the pointer internal to the
            // bootloader receive state machine. This is here because we may
            // have received two commands in the same buffer and we want to
            // handle them both back-to-back.
            if need_reset {
                decoder.reset();
                need_reset = false;
            }

            match decoder.receive(buffer[i]) {
                Ok(None) => {}
                Ok(Some(tock_bootloader_protocol::Command::Ping)) => {
                    self.buffer.replace(buffer);
                    self.send_response(RES_PONG);
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::Reset)) => {
                    need_reset = true;
                    // If there are more bytes in the buffer we want to continue
                    // parsing those. Otherwise, we want to go back to receive.
                    if i == rx_len - 1 {
                        self.uart
                            .receive_automatic(buffer, buffer.len(), UART_RECEIVE_TIMEOUT);
                        break;
                    }
                }
                Ok(Some(tock_bootloader_protocol::Command::Info)) => {
                    self.state.set(State::Info);
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        // Calculate the page index given that flags start
                        // at address 1024.
                        let page_index = self.flags_address / page.as_mut().len();

                        let _ = self.flash.read_page(page_index, page);
                    });
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::ReadRange { address, length })) => {
                    self.state.set(State::ReadRange {
                        address,
                        length,
                        remaining_length: length,
                    });
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        let page_size = page.as_mut().len();
                        let _ = self.flash.read_page(address as usize / page_size, page);
                    });
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::WritePage { address, data })) => {
                    self.page_buffer.take().map(move |page| {
                        let page_size = page.as_mut().len();
                        if page_size != data.len() {
                            // Error if we didn't get exactly a page of data
                            // to write to flash.
                            buffer[0] = ESCAPE_CHAR;
                            buffer[1] = RES_BADARGS;
                            self.page_buffer.replace(page);
                            self.state.set(State::Idle);
                            self.uart.transmit_buffer(buffer, 2);
                        } else if address >= self.bootloader_address
                            && address < self.bootloader_end_address
                        {
                            // Do not allow the bootloader to try to overwrite
                            // itself. This will largely not work, and would be
                            // irreversible for the user.
                            buffer[0] = ESCAPE_CHAR;
                            buffer[1] = RES_BADADDR;
                            self.page_buffer.replace(page);
                            self.state.set(State::Idle);
                            self.uart.transmit_buffer(buffer, 2);
                        } else {
                            // Otherwise copy into page buffer and write to
                            // flash.
                            for i in 0..page_size {
                                page.as_mut()[i] = data[i];
                            }
                            self.state.set(State::WriteFlashPage);
                            self.buffer.replace(buffer);
                            let _ = self.flash.write_page(address as usize / page_size, page);
                        }
                    });
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::ErasePage { address })) => {
                    self.state.set(State::ErasePage);
                    self.buffer.replace(buffer);
                    let page_size = self.page_buffer.map_or(512, |page| page.as_mut().len());
                    let _ = self.flash.erase_page(address as usize / page_size);
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::CrcIntFlash { address, length })) => {
                    self.state.set(State::Crc {
                        address,
                        remaining_length: length,
                        crc: 0xFFFFFFFF,
                    });
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        let page_size = page.as_mut().len();
                        let _ = self.flash.read_page(address as usize / page_size, page);
                    });
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::GetAttr { index })) => {
                    self.state.set(State::GetAttribute { index: index });
                    self.buffer.replace(buffer);
                    self.page_buffer.take().map(move |page| {
                        // Need to calculate which page to read to get the
                        // correct attribute (each attribute is 64 bytes long),
                        // where attributes start at address 0x600.
                        let page_len = page.as_mut().len();
                        let read_address = self.attributes_address + (index as usize * 64);
                        let page_index = read_address / page_len;

                        let _ = self.flash.read_page(page_index, page);
                    });
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::SetAttr { index, key, value })) => {
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
                        // Need to calculate which page to read to get the
                        // correct attribute (each attribute is 64 bytes long),
                        // where attributes start at address 0x600.
                        let page_len = page.as_mut().len();
                        let read_address = self.attributes_address + (index as usize * 64);
                        let page_index = read_address / page_len;

                        let _ = self.flash.read_page(page_index, page);
                    });
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::SetStartAddress { address })) => {
                    self.state.set(State::SetStartAddress { address });
                    self.buffer.replace(buffer);

                    // Initiate things by reading the correct flash page that
                    // needs to be updated.
                    self.page_buffer.take().map(move |page| {
                        let page_len = page.as_mut().len();
                        let page_index = self.flags_address / page_len;

                        let _ = self.flash.read_page(page_index, page);
                    });
                    break;
                }
                Ok(Some(tock_bootloader_protocol::Command::Exit)) => {
                    (self.reset_function)();
                    break;
                }
                Ok(Some(_)) => {
                    self.buffer.replace(buffer);
                    self.send_response(RES_UNKNOWN);
                    break;
                }
                Err(tock_bootloader_protocol::Error::BadArguments) => {
                    self.buffer.replace(buffer);
                    self.send_response(RES_BADARGS);
                    break;
                }
                Err(_) => {
                    self.buffer.replace(buffer);
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
        }
    }
}

impl<'a, U: hil::uart::UartAdvanced<'a> + 'a, F: hil::flash::Flash + 'a> hil::flash::Client<F>
    for Bootloader<'a, U, F>
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
                    let mut index = 3;

                    // Insert the first part of the JSON blob into the buffer.
                    let str01 = "{\"version\":\"";
                    for i in 0..str01.len() {
                        buffer[index] = str01.as_bytes()[i];
                        index += 1;
                    }

                    // Calculate where in the page the flags start.
                    let page_offset = self.flags_address % pagebuffer.as_mut().len();

                    // Version string is at most 8 bytes long, and starts
                    // at index 14 in the bootloader page.
                    for i in 0..8 {
                        let b = pagebuffer.as_mut()[i + 14 + page_offset];
                        if b == 0 {
                            break;
                        }
                        buffer[index] = b;
                        index += 1;
                    }

                    // Do start address
                    let str02 = "\", \"start_address\":\"0x";
                    for i in 0..str02.len() {
                        buffer[index] = str02.as_bytes()[i];
                        index += 1;
                    }
                    for i in 0..8 {
                        let b = (pagebuffer.as_mut()[32 + page_offset + 3 - (i / 2)]
                            >> (((i + 1) % 2) * 4))
                            & 0x0F;
                        buffer[index] = char::from_digit(b.into(), 16).unwrap_or('?') as u8;
                        index += 1;
                    }
                    let str02 = "\", ";
                    for i in 0..str02.len() {
                        buffer[index] = str02.as_bytes()[i];
                        index += 1;
                    }

                    // Insert the last half of the JSON blob into the buffer.
                    let str02 = "\"name\":\"Tock Bootloader\"}";
                    for i in 0..str02.len() {
                        buffer[index] = str02.as_bytes()[i];
                        index += 1;
                    }

                    // Need to insert the string length as the first byte
                    // after the header.
                    buffer[2] = index as u8 - 3;
                    index += 1;

                    // Rest should be 0.
                    for i in index..195 {
                        buffer[i] = 0;
                    }

                    self.page_buffer.replace(pagebuffer);
                    self.uart.transmit_buffer(buffer, 195);
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

                    // Need to calculate where in the page to look for this
                    // attribute with attributes starting at address 0x600 and
                    // where each has length of 64 bytes.
                    let page_len = pagebuffer.as_mut().len();
                    let read_address = self.attributes_address + (index as usize * 64);
                    let page_offset = read_address % page_len;

                    for i in 0..64 {
                        let b = pagebuffer.as_mut()[page_offset + i];
                        if b == ESCAPE_CHAR {
                            // Need to escape the escape character.
                            buffer[j] = ESCAPE_CHAR;
                            j += 1;
                        }
                        buffer[j] = b;
                        j += 1;
                    }

                    self.page_buffer.replace(pagebuffer);
                    self.uart.transmit_buffer(buffer, j);
                });
            }

            // We need to update the page we just read with the new attribute,
            // and then write that all back to flash.
            State::SetAttribute { index } => {
                self.buffer.map(move |buffer| {
                    let page_len = pagebuffer.as_mut().len();
                    let read_address = self.attributes_address + (index as usize * 64);
                    let page_offset = read_address % page_len;
                    let page_index = read_address / page_len;

                    // Copy the first 64 bytes of the buffer into the correct
                    // spot in the page.
                    for i in 0..64 {
                        pagebuffer.as_mut()[page_offset + i] = buffer[i];
                    }
                    let _ = self.flash.write_page(page_index, pagebuffer);
                });
            }

            // We need to update the page we just read with the new attribute,
            // and then write that all back to flash.
            State::SetStartAddress { address } => {
                let page_len = pagebuffer.as_mut().len();
                let read_address = self.flags_address + 32;
                let page_offset = read_address % page_len;
                let page_index = read_address / page_len;

                // Copy the first 64 bytes of the buffer into the correct
                // spot in the page.
                for (i, v) in address.to_le_bytes().iter().enumerate() {
                    pagebuffer.as_mut()[page_offset + i] = *v;
                }
                let _ = self.flash.write_page(page_index, pagebuffer);
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
                    self.uart.transmit_buffer(buffer, index);
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
                        self.uart.transmit_buffer(buffer, 6);
                    });
                } else {
                    // More CRC to do!
                    self.state.set(State::Crc {
                        address: new_address,
                        remaining_length: new_remaining_length,
                        crc: new_crc,
                    });
                    let _ = self
                        .flash
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
                    self.uart.transmit_buffer(buffer, 2);
                });
            }

            // Attribute writing done, send an OK response.
            State::SetAttribute { index: _ } => {
                self.state.set(State::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_OK;
                    self.uart.transmit_buffer(buffer, 2);
                });
            }

            // Attribute writing done, send an OK response.
            State::SetStartAddress { address: _ } => {
                self.state.set(State::Idle);
                self.buffer.take().map(move |buffer| {
                    buffer[0] = ESCAPE_CHAR;
                    buffer[1] = RES_OK;
                    self.uart.transmit_buffer(buffer, 2);
                });
            }

            _ => {
                self.buffer.take().map(|buffer| {
                    self.uart
                        .receive_automatic(buffer, buffer.len(), UART_RECEIVE_TIMEOUT);
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
                    self.uart.transmit_buffer(buffer, 2);
                });
            }

            _ => {
                self.buffer.take().map(|buffer| {
                    self.uart
                        .receive_automatic(buffer, buffer.len(), UART_RECEIVE_TIMEOUT);
                });
            }
        }
    }
}
