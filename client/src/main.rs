use std::{net::TcpStream, thread};

use sbmp::read::FrameReader;
use sbmp::write::FrameWriter;
use std::io;

fn main() {
    let stream = TcpStream::connect("0.0.0.0:1337").expect("Could not connect to the server");
    let reader = stream.try_clone().expect("Could not clone stream");

    let reader = FrameReader::new(reader);
    let writer = FrameWriter::new(stream);

    thread::spawn(move || {
        let msg = new_message();
        msg.as_bytes()
    });
}

fn new_message() -> String {
    let mut s = String::new();

    let _ = io::stdin().read_line(&mut s).expect("Stdin error");

    s
}
