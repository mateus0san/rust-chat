// Simple Binary Messaging Protocol
// +------------+------------+------------+----------------+
// | 1 byte     | 1 byte     | 4 bytes    | N bytes        |
// | version    | type       | length     | payload        |
// +------------+------------+------------+----------------+

pub mod sbmp;

use crate::sbmp::{ContentType, Frame, Header, SBMPError};
use std::io::{Read, Write};

pub mod write {
    use super::*;

    pub struct FrameWriter<W: Write> {
        writer: W,
    }

    impl<W: Write> FrameWriter<W> {
        pub fn new(stream: W) -> Self {
            Self { writer: stream }
        }

        pub fn write_frame(&mut self, frame: Frame) -> Result<(), SBMPError> {
            let (header, payload) = frame.get();
            let mut data: Vec<u8> = Vec::new();
            let content_len = header.content_len() as u32;

            data.push(header.version());
            data.push(header.content_type() as u8);
            data.extend(content_len.to_be_bytes().iter());

            self.writer.write_all(&data)?;
            self.writer.write_all(&payload)?;

            Ok(())
        }
    }
}

pub mod read {
    use super::*;

    pub struct FrameReader<R: Read> {
        reader: R,
    }

    impl<R: Read> FrameReader<R> {
        pub fn new(stream: R) -> Self {
            Self { reader: stream }
        }

        pub fn read_frame(&mut self) -> Result<Frame, SBMPError> {
            let header = self.read_header()?;

            let mut payload = vec![0u8; header.content_len()];
            self.reader.read_exact(&mut payload)?;

            Frame::try_new(header, payload)
        }

        fn read_header(&mut self) -> Result<Header, SBMPError> {
            // version, 1 byte
            let mut byte = [0u8; 1];
            self.reader.read_exact(&mut byte)?;
            let version = byte[0];

            // content-type, 1 byte
            let mut content_type = [0u8; 1];
            self.reader.read_exact(&mut content_type)?;
            let content_type = ContentType::try_from(content_type[0])?;

            // content-length, 4 bytes
            let mut content_length = [0u8; 4];
            self.reader.read_exact(&mut content_length)?;
            let content_length = u32::from_be_bytes(content_length);

            Header::try_new(version, content_type, content_length)
        }
    }
}
