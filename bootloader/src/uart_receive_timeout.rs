//! Convert the normal UART interface to one with a timeout.

// use core::cmp;
use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::hil::time::Frequency;
use kernel::ReturnCode;



// pub struct UartReceiveTimeout<'a, U: hil::uart::UART + 'a> {
pub struct UartReceiveTimeout<'a, A: hil::time::Alarm + 'a> {
    uart: &'static hil::uart::UART,
    alarm: &'a A,
    rx_pin: &'a hil::gpio::Pin,
    client: OptionalCell<&'static hil::uart::Client>,
    // tx_buffer: TakeCell<'static, [u8]>,
    // rx_buffer: TakeCell<'static, [u8]>,
}

impl<'a, A: hil::time::Alarm> UartReceiveTimeout<'a, A> {
    pub fn new(
        uart: &'static hil::uart::UART,
        alarm: &'a A,
        rx_pin: &'a hil::gpio::Pin,
        // tx_buffer: &'static mut [u8],
        // rx_buffer: &'static mut [u8],
    // ) -> UartReceiveTimeout<'a, U> {
    ) -> UartReceiveTimeout<'a, A> {
        UartReceiveTimeout {
            uart: uart,
            alarm: alarm,
            rx_pin: rx_pin,
            client: OptionalCell::empty(),
            // tx_buffer: TakeCell::new(tx_buffer),
            // rx_buffer: TakeCell::new(rx_buffer),
        }
    }

    pub fn initialize(&self) {
        // self.uart.configure(hil::uart::UARTParameters {
        //     baud_rate: 250000,
        //     stop_bits: hil::uart::StopBits::One,
        //     parity: hil::uart::Parity::Even,
        //     hw_flow_control: true,
        // });

        self.rx_pin.make_input();
        self.rx_pin.enable_interrupt(0, hil::gpio::InterruptMode::FallingEdge);
    }
}

// impl<'a, U: hil::uart::UART> hil::uart::UART for UartReceiveTimeout<'a, U> {
impl<'a, A: hil::time::Alarm> hil::uart::UART for UartReceiveTimeout<'a, A> {
    fn set_client(&self, client: &'static hil::uart::Client) {
        self.client.set(client);
    }

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

// impl<'a, U: hil::uart::UART> hil::uart::UARTReceiveAdvanced for UartReceiveTimeout<'a, U> {
impl<'a, A: hil::time::Alarm> hil::uart::UARTReceiveAdvanced for UartReceiveTimeout<'a, A> {
    fn receive_automatic(&self, rx_buffer: &'static mut [u8], interbyte_timeout: u8) {
        // let interval = (20 as u32) * <A::Frequency>::frequency() / 1000;

        // let tics = self.alarm.now().wrapping_add(interval);
        // self.alarm.set_alarm(tics);



        // debug_gpio!(0, toggle);
        let len = rx_buffer.len();
        self.uart.receive(rx_buffer, len);
    }
}

impl<'a, A: hil::time::Alarm> hil::gpio::Client for UartReceiveTimeout<'a, A> {
    // This is called when the UART RX pin toggles.
    // We need to stop any existing timers and set a new timer to see if this
    // is the last byte.
    fn fired(&self, _: usize) {
        // self.client.map(|client| {
        //     client.interrupt();
        // });
        //
        let interval = (20 as u32) * <A::Frequency>::frequency() / 1000;
        let tics = self.alarm.now().wrapping_add(interval);
        self.alarm.set_alarm(tics);
    }
}

impl<'a, A: hil::time::Alarm> hil::time::Client for UartReceiveTimeout<'a, A> {
    fn fired(&self) {
        // self.buffer.take().map(|buffer| {
        //     // turn on i2c to send commands
        //     self.i2c.enable();

        //     self.i2c.read(buffer, 2);
        //     match self.state.get() {
        //         State::WaitRh => self.state.set(State::ReadRhMeasurement),
        //         State::WaitTemp => self.state.set(State::ReadTempMeasurement),
        //         _ => (),
        //     }
        // });

        self.uart.abort_receive();
    }
}

// Callbacks from the underlying UART driver.
// impl<'a, U: hil::uart::UART> hil::uart::Client for UartReceiveTimeout<'a, U> {
impl<'a, A: hil::time::Alarm> hil::uart::Client for UartReceiveTimeout<'a, A> {
    // Called when the UART TX has finished.
    fn transmit_complete(&self, buffer: &'static mut [u8], error: hil::uart::Error) {
        self.client.map(move |client| {
            client.transmit_complete(buffer, error);
        });
    }

    // Called when a buffer is received on the UART.
    fn receive_complete(&self, buffer: &'static mut [u8], rx_len: usize, error: hil::uart::Error) {
        // self.rx_buffer.replace(buffer);
        // self.client.receive_complete(buffer, rx_len, error);

        self.client.map(move |client| {
            client.receive_complete(buffer, rx_len, error);
        });

        // self.app.map(|appst| {
        //     appst.rx_buffer = appst.rx_buffer.take().map(|mut rb| {
        //         // Figure out length to copy.
        //         let max_len = cmp::min(rx_len, rb.len());

        //         // Copy over data to app buffer.
        //         self.rx_buffer.map(|buffer| {
        //             for idx in 0..max_len {
        //                 rb.as_mut()[idx] = buffer[idx];
        //             }
        //         });
        //         appst.callback.as_mut().map(|cb| {
        //             // Notify the serialization library in userspace about the
        //             // received buffer.
        //             cb.schedule(4, rx_len, 0);
        //         });

        //         rb
        //     });
        // });

        // // Restart the UART receive.
        // self.rx_buffer
        //     .take()
        //     .map(|buffer| self.uart.receive_automatic(buffer, 250));
    }
}
