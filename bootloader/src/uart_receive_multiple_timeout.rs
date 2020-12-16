//! Convert the normal UART interface to one with a timeout.
//!
//! This capsule provides the `hil::uart::ReceiveAdvanced` interface, and for
//! the most part just passes it through to the underlying `uart` peripheral.
//! However, it also provides a DIY version of `receive_automatic()` for
//! platforms where the hardware does not provide it natively.
//!
//! It does this by calling `uart.receive_buffer()` multiple times, where
//! the second call and every call after that also starts a timer. If the timer
//! expires the receive is aborted and the receive finishes.
//!
//!

use core::cell::Cell;
use core::cmp;

use kernel::common::cells::OptionalCell;
use kernel::common::cells::TakeCell;
use kernel::hil;
use kernel::ReturnCode;

pub static mut BUF: [u8; 512] = [0; 512];

#[derive(Copy, Clone, PartialEq)]
enum State {
    Idle,
    Receiving,
}

pub struct UartReceiveMultipleTimeout<'a, A: hil::time::Alarm<'a> + 'a> {
    uart: &'a dyn hil::uart::Uart<'a>,
    alarm: &'a A,
    rx_buffer: TakeCell<'static, [u8]>,

    rx_client: OptionalCell<&'a dyn hil::uart::ReceiveClient>,
    rx_client_buffer: TakeCell<'static, [u8]>,
    rx_client_index: Cell<usize>,

    state: Cell<State>,
}

impl<'a, A: hil::time::Alarm<'a>> UartReceiveMultipleTimeout<'a, A> {
    pub fn new(
        uart: &'a dyn hil::uart::Uart<'a>,
        alarm: &'a A,
        rx_buffer: &'static mut [u8],
    ) -> UartReceiveMultipleTimeout<'a, A> {
        UartReceiveMultipleTimeout {
            uart,
            alarm,
            rx_client: OptionalCell::empty(),
            rx_buffer: TakeCell::new(rx_buffer),
            rx_client_buffer: TakeCell::empty(),
            rx_client_index: Cell::new(0),
            state: Cell::new(State::Idle),
        }
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::uart::Configure for UartReceiveMultipleTimeout<'a, A> {
    fn configure(&self, params: hil::uart::Parameters) -> ReturnCode {
        self.uart.configure(params)
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::uart::Receive<'a> for UartReceiveMultipleTimeout<'a, A> {
    fn set_receive_client(&self, client: &'a dyn hil::uart::ReceiveClient) {
        self.rx_client.set(client);
    }

    fn receive_buffer(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        self.uart.receive_buffer(rx_buffer, rx_len)
    }

    fn receive_word(&self) -> ReturnCode {
        self.uart.receive_word()
    }

    fn receive_abort(&self) -> ReturnCode {
        self.uart.receive_abort()
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::uart::Transmit<'a> for UartReceiveMultipleTimeout<'a, A> {
    fn set_transmit_client(&self, _client: &'a dyn hil::uart::TransmitClient) {}

    fn transmit_buffer(
        &self,
        tx_buffer: &'static mut [u8],
        tx_len: usize,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        self.uart.transmit_buffer(tx_buffer, tx_len)
    }

    fn transmit_word(&self, word: u32) -> ReturnCode {
        self.uart.transmit_word(word)
    }

    fn transmit_abort(&self) -> ReturnCode {
        self.uart.transmit_abort()
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::uart::ReceiveAdvanced<'a>
    for UartReceiveMultipleTimeout<'a, A>
{
    fn receive_automatic(
        &self,
        rx_buffer: &'static mut [u8],
        _rx_len: usize,
        _interbyte_timeout: u8,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        match self.state.get() {
            State::Idle => {
                // Nothing is happening with receive right now. So, all we do
                // is start a new receive, and wait for that to finish.
                self.state.set(State::Receiving);

                // We save the client's buffer.
                self.rx_client_buffer.replace(rx_buffer);

                // Reset the index counter to 0 since we starting a new receive.
                self.rx_client_index.set(0);

                // We want to ensure that we always get a callback when anything
                // is received, so we ask for 1 byte. We may get more than this.
                self.rx_buffer
                    .take()
                    .map(|rx| self.uart.receive_buffer(rx, 1));

                (ReturnCode::SUCCESS, None)
            }

            State::Receiving => {
                // We are in the middle of a receive. We cannot start another
                // receive at this point.
                (ReturnCode::EBUSY, Some(rx_buffer))
            }
        }
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::uart::UartAdvanced<'a>
    for UartReceiveMultipleTimeout<'a, A>
{}

impl<'a, A: hil::time::Alarm<'a>> hil::time::AlarmClient for UartReceiveMultipleTimeout<'a, A> {
    /// If the timer actually fires then we stopped receiving bytes.
    fn alarm(&self) {
        // Cancel the receive so that we get the buffer back.
        self.uart.receive_abort();
    }
}

// Callbacks from the underlying UART driver.
impl<'a, A: hil::time::Alarm<'a>> hil::uart::TransmitClient for UartReceiveMultipleTimeout<'a, A> {
    // Called when the UART TX has finished.
    fn transmitted_buffer(&self, _buffer: &'static mut [u8], _tx_len: usize, _rval: ReturnCode) {}
}

// Callbacks from the underlying UART driver.
impl<'a, A: hil::time::Alarm<'a>> hil::uart::ReceiveClient for UartReceiveMultipleTimeout<'a, A> {
    // Called when a buffer is received on the UART.
    fn received_buffer(
        &self,
        buffer: &'static mut [u8],
        rx_len: usize,
        rval: ReturnCode,
        _error: hil::uart::Error,
    ) {
        match self.state.get() {
            State::Idle => {
                // Can't get here.
            }

            State::Receiving => {
                // We got the first payload from the underlying receive channel.

                // First we always copy what we just received into the client's
                // buffer.
                self.rx_client_buffer.map(|client_rx| {
                    let rx_offset = self.rx_client_index.get();

                    // How many more bytes can we store in our RX buffer?
                    let available_bytes = client_rx.len() - rx_offset;
                    let copy_length = cmp::min(rx_len, available_bytes);

                    // Do the copy into the RX buffer.
                    for i in 0..copy_length {
                        client_rx[rx_offset + i] = buffer[i];
                    }
                    self.rx_client_index.set(rx_offset + copy_length);
                });

                // If everything is normal then we continue receiving.
                if rval == ReturnCode::SUCCESS {
                    // Next we setup a timer to timeout if the receive has
                    // finished.
                    let interval = A::ticks_from_ms(4);
                    self.alarm.set_alarm(self.alarm.now(), interval);

                    // Then we go back to receiving to see if there is more data
                    // on its way.
                    //
                    // Receive up to half of the buffer at a time so there is
                    // room if the host sends us more than we expect.
                    self.uart.receive_buffer(buffer, buffer.len() / 2);
                } else if rval == ReturnCode::ECANCEL {
                    // The last receive was aborted meaning the receive has
                    // finished.

                    // Replace our buffer.
                    self.rx_buffer.replace(buffer);

                    // We are no longer receiving.
                    self.state.set(State::Idle);

                    // Call receive complete to the client.
                    self.rx_client.map(|client| {
                        self.rx_client_buffer.take().map(|rx_buffer| {
                            client.received_buffer(
                                rx_buffer,
                                self.rx_client_index.get(),
                                ReturnCode::SUCCESS,
                                hil::uart::Error::None,
                            );
                        });
                    });
                }
            }
        }
    }
}
