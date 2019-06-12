//! Convert the normal UART interface to one with a timeout.
//!
//! This capsule provides the `hil::uart::UART` interface, and for the most part
//! just passes it through to the underlying `uart` peripheral. However, it
//! also provides a DIY version of `receive_automatic()` for platforms where
//! the hardware does not provide it natively.
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
use kernel::hil::time::Frequency;
use kernel::ReturnCode;

pub struct UartReceiveTimeout<'a, A: hil::time::Alarm + 'a> {
    uart: &'static hil::uart::UART,
    alarm: &'a A,
    rx_pin: &'a hil::gpio::Pin,
}

impl<'a, A: hil::time::Alarm> UartReceiveTimeout<'a, A> {
    pub fn new(
        uart: &'static hil::uart::UART,
        alarm: &'a A,
        rx_pin: &'a hil::gpio::Pin,
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
            .enable_interrupt(0, hil::gpio::InterruptMode::FallingEdge);
    }
}

impl<'a, A: hil::time::Alarm> hil::uart::UART for UartReceiveTimeout<'a, A> {
    fn set_client(&self, _client: &'static hil::uart::Client) {}

    fn configure(&self, params: hil::uart::UARTParameters) -> ReturnCode {
        self.uart.configure(params)
    }

    fn transmit(&self, tx_data: &'static mut [u8], tx_len: usize) {
        self.uart.transmit(tx_data, tx_len);
    }

    fn receive(&self, rx_buffer: &'static mut [u8], rx_len: usize) {
        self.uart.receive(rx_buffer, rx_len);
    }

    fn abort_receive(&self) {
        self.uart.abort_receive();
    }
}

impl<'a, A: hil::time::Alarm> hil::uart::UARTReceiveAdvanced for UartReceiveTimeout<'a, A> {
    fn receive_automatic(&self, rx_buffer: &'static mut [u8], _interbyte_timeout: u8) {
        // Just call receive with the entire buffer.
        let len = rx_buffer.len();
        self.uart.receive(rx_buffer, len);
    }
}

impl<'a, A: hil::time::Alarm> hil::gpio::Client for UartReceiveTimeout<'a, A> {
    // This is called when the UART RX pin toggles.
    // We start a new timer on every toggle to wait for the end of incoming
    // RX bytes.
    fn fired(&self, _: usize) {
        let interval = (30 as u32) * <A::Frequency>::frequency() / 1000;
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }
}

impl<'a, A: hil::time::Alarm> hil::time::Client for UartReceiveTimeout<'a, A> {
    /// If the timer actually fires then we stopped receiving bytes.
    fn fired(&self) {
        self.uart.abort_receive();
    }
}

// Callbacks from the underlying UART driver.
impl<'a, A: hil::time::Alarm> hil::uart::Client for UartReceiveTimeout<'a, A> {
    // Called when the UART TX has finished.
    fn transmit_complete(&self, _buffer: &'static mut [u8], _error: hil::uart::Error) {}

    // Called when a buffer is received on the UART.
    fn receive_complete(
        &self,
        _buffer: &'static mut [u8],
        _rx_len: usize,
        _error: hil::uart::Error,
    ) {

    }
}
