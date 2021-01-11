# Tockloader Protocol

Implements the Tock bootloader over-the-wire protocol.

Originally from: https://github.com/thejpster/tockloader-proto-rs


Usage
-----

In your embedded bootloader, you need a loop that looks something like:

```rust
use tockloader_proto::{ResponseEncoder, CommandDecoder};

#[no_mangle]
pub extern "C" fn main() {
    let mut uart = uart::Uart::new(uart::UartId::Uart0, 115200, uart::NewlineMode::Binary);
    let mut decoder = CommandDecoder::new();
    loop {
        if let Ok(Some(ch)) = uart.getc_try() {
            let mut need_reset = false;
            let response = match decoder.receive(ch) {
                Ok(None) => None,
                Ok(Some(tockloader_proto::Command::Ping)) => Some(tockloader_proto::Response::Pong),
                Ok(Some(tockloader_proto::Command::Reset)) => {
                    need_reset = true;
                    None
                },
                Ok(Some(_)) => Some(tockloader_proto::Response::Unknown),
                Err(_) => Some(tockloader_proto::Response::InternalError),
            };
            if need_reset {
                decoder.reset();
            }
            if let Some(response) = response {
                let mut encoder = ResponseEncoder::new(&response).unwrap();
                while let Some(byte) = encoder.next() {
                    uart.putc(byte);
                }
            }
        }
    }
}
```

Using this library in a CLI flash tool (like tockloader) is left as an excercise
for the read (hint: you want `ResponseDecoder` and `CommandEncoder`).
