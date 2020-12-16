extern crate tockloader_proto;

use tockloader_proto::prelude::*;

fn main() {
    let r = tockloader_proto::Response::Pong;
    let mut e = tockloader_proto::ResponseEncoder::new(&r).unwrap();
    let mut buffer = [0xFFu8; 4];
    let used = e.write(&mut buffer);
    println!("Buffer: {:?}", &buffer[0..used]);
}
