use std::{net::TcpStream, thread};

use sbmp::read::FrameReader;
use sbmp::sbmp::ContentType;
use sbmp::write::{FrameWriter, build_frame};
use std::io;

fn main() {
    let stream = TcpStream::connect("0.0.0.0:1337").expect("Could not connect to the server");
    let reader = stream.try_clone().expect("Could not clone stream");

    let mut reader = FrameReader::new(reader);
    let mut writer = FrameWriter::new(stream);

    thread::spawn(move || {
        loop {
            let msg = new_message();
            let frame = build_frame(ContentType::UTF8, msg.as_bytes())
                .expect("I need to remove this expect later");
            writer.write_frame(frame).expect("remove this expect later");
        }
    });

    loop {
        let frame = match reader.read_frame() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("{:#?}", e);
                continue;
            }
        };
        let payload = String::from_utf8(frame.get_payload()).expect("remove this expect later");
        println!("{payload}");
    }
}

fn new_message() -> String {
    let mut s = String::new();

    let _ = io::stdin().read_line(&mut s).expect("Stdin error");

    s
}
