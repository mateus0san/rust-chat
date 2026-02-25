use crate::sbmp::{Header, SBMPError};

pub struct Frame {
    header: Header,
    payload: Vec<u8>,
}

impl Frame {
    fn new(header: Header, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    pub fn try_new(header: Header, payload: Vec<u8>) -> Result<Self, SBMPError> {
        if header.content_len() == payload.len() {
            Ok(Frame::new(header, payload))
        } else {
            Err(SBMPError::ContentLenDiff)
        }
    }

    pub fn get(&self) -> (&Header, &Vec<u8>) {
        (&self.header, &self.payload)
    }
}
