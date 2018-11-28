//! Convert the normal UART interface to one with a timeout.

// use core::cmp;
use kernel::common::cells::OptionalCell;
use kernel::hil;
use kernel::ReturnCode;



// pub struct UartReceiveTimeout<'a, U: hil::uart::UART + 'a> {
pub struct UartReceiveTimeout {
    uart: &'static hil::uart::UART,
    client: OptionalCell<&'static hil::uart::Client>,
    // tx_buffer: TakeCell<'static, [u8]>,
    // rx_buffer: TakeCell<'static, [u8]>,
}

impl UartReceiveTimeout {
    pub fn new(
        uart: &'static hil::uart::UART,
        // tx_buffer: &'static mut [u8],
        // rx_buffer: &'static mut [u8],
    // ) -> UartReceiveTimeout<'a, U> {
    ) -> UartReceiveTimeout {
        UartReceiveTimeout {
            uart: uart,
            client: OptionalCell::empty(),
            // tx_buffer: TakeCell::new(tx_buffer),
            // rx_buffer: TakeCell::new(rx_buffer),
        }
    }

    // pub fn initialize(&self) {
    //     self.uart.configure(hil::uart::UARTParameters {
    //         baud_rate: 250000,
    //         stop_bits: hil::uart::StopBits::One,
    //         parity: hil::uart::Parity::Even,
    //         hw_flow_control: true,
    //     });
    // }
}

// impl<'a, U: hil::uart::UART> hil::uart::UART for UartReceiveTimeout<'a, U> {
impl hil::uart::UART for UartReceiveTimeout {
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
impl hil::uart::UARTReceiveAdvanced for UartReceiveTimeout {
    fn receive_automatic(&self, rx_buffer: &'static mut [u8], interbyte_timeout: u8) {
        let len = rx_buffer.len();
        self.uart.receive(rx_buffer, len);
    }
}

// Callbacks from the underlying UART driver.
// impl<'a, U: hil::uart::UART> hil::uart::Client for UartReceiveTimeout<'a, U> {
impl hil::uart::Client for UartReceiveTimeout {
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
