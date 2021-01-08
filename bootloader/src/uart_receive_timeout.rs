//! Convert the normal UART interface to one with a timeout.
//!
//! This capsule provides the `hil::uart::ReceiveAdvanced` interface, and for
//! the most part just passes it through to the underlying `uart` peripheral.
//! However, it also provides a DIY version of `receive_automatic()` for
//! platforms where the hardware does not provide it natively.
//!
//! It does this by using the UART RX pin as an interrupt source, and a timer
//! to wait for the end of received bytes. On each interrupt from the UART bytes
//! the timer is reset, and when the timer finally fires then `abort_receive()`
//! is called to stop the receive.
//!
//! This module doesn't do anything on the UART client side, so the client
//! of the underlying uart driver should be set to the upper layer.
//!
//! Usage
//! -----
//!
//! ```
//! let recv_auto_virtual_alarm = static_init!(
//!     VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>,
//!     VirtualMuxAlarm::new(mux_alarm)
//! );
//!
//! let recv_auto_uart = static_init!(
//!     bootloader::uart_receive_timeout::UartReceiveTimeout<'static, VirtualMuxAlarm<'static, nrf5x::rtc::Rtc>>,
//!     bootloader::uart_receive_timeout::UartReceiveTimeout::new(&nrf52::uart::UARTE0,
//!         recv_auto_virtual_alarm,
//!         &nrf5x::gpio::PORT[UART_RXD]
//!         )
//!     );
//! recv_auto_virtual_alarm.set_client(recv_auto_uart);
//! nrf5x::gpio::PORT[UART_RXD].set_client(recv_auto_uart);
//! recv_auto_uart.initialize();
//! ```

use kernel::hil;
use kernel::ReturnCode;

pub struct UartReceiveTimeout<'a, A: hil::time::Alarm<'a> + 'a> {
    uart: &'a dyn hil::uart::UartData<'a>,
    alarm: &'a A,
    rx_pin: &'a dyn hil::gpio::InterruptPin<'a>,
}

impl<'a, A: hil::time::Alarm<'a>> UartReceiveTimeout<'a, A> {
    pub fn new(
        uart: &'a dyn hil::uart::UartData<'a>,
        alarm: &'a A,
        rx_pin: &'a dyn hil::gpio::InterruptPin<'a>,
    ) -> UartReceiveTimeout<'a, A> {
        UartReceiveTimeout {
            uart: uart,
            alarm: alarm,
            rx_pin: rx_pin,
        }
    }

    /// Setup the GPIO interrupt to wait for the end of UART bytes.
    pub fn initialize(&self) {
        self.rx_pin.make_input();
        self.rx_pin
            .enable_interrupts(hil::gpio::InterruptEdge::FallingEdge);
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::uart::Transmit<'a> for UartReceiveTimeout<'a, A> {
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

impl<'a, A: hil::time::Alarm<'a>> hil::uart::Receive<'a> for UartReceiveTimeout<'a, A> {
    fn set_receive_client(&self, _client: &'a dyn hil::uart::ReceiveClient) {}

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

impl<'a, A: hil::time::Alarm<'a>> hil::uart::ReceiveAdvanced<'a> for UartReceiveTimeout<'a, A> {
    fn receive_automatic(
        &self,
        rx_buffer: &'static mut [u8],
        rx_len: usize,
        _interbyte_timeout: u8,
    ) -> (ReturnCode, Option<&'static mut [u8]>) {
        // Just call receive with the entire buffer.
        self.uart.receive_buffer(rx_buffer, rx_len)
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::gpio::Client for UartReceiveTimeout<'a, A> {
    // This is called when the UART RX pin toggles.
    // We start a new timer on every toggle to wait for the end of incoming
    // RX bytes.
    fn fired(&self) {
        let interval = A::ticks_from_ms(30);
        self.alarm.set_alarm(self.alarm.now(), interval);
    }
}

impl<'a, A: hil::time::Alarm<'a>> hil::time::AlarmClient for UartReceiveTimeout<'a, A> {
    /// If the timer actually fires then we stopped receiving bytes.
    fn alarm(&self) {
        self.uart.receive_abort();
    }
}

// Callbacks from the underlying UART driver.
impl<'a, A: hil::time::Alarm<'a>> hil::uart::TransmitClient for UartReceiveTimeout<'a, A> {
    // Called when the UART TX has finished.
    fn transmitted_buffer(&self, _buffer: &'static mut [u8], _tx_len: usize, _rval: ReturnCode) {}
}

// Callbacks from the underlying UART driver.
impl<'a, A: hil::time::Alarm<'a>> hil::uart::ReceiveClient for UartReceiveTimeout<'a, A> {
    // Called when a buffer is received on the UART.
    fn received_buffer(
        &self,
        _buffer: &'static mut [u8],
        _rx_len: usize,
        _rval: ReturnCode,
        _error: hil::uart::Error,
    ) {
    }
}
