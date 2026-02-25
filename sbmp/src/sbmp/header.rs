use crate::sbmp::{ContentType, PROTOCOL_VERSION, SBMPError};

pub struct Header {
    version: u8,
    content_type: ContentType,
    content_length: usize,
}

impl Header {
    fn new(version: u8, content_type: ContentType, content_length: usize) -> Self {
        Self {
            version,
            content_type,
            content_length,
        }
    }

    pub fn try_new(
        version: u8,
        content_type: ContentType,
        content_length: u32,
    ) -> Result<Self, SBMPError> {
        if version != PROTOCOL_VERSION {
            return Err(SBMPError::WrongVersion);
        }

        const KB: u32 = 1024; // KiloBytes 
        const MB: u32 = KB * 1024; // MegaBytes

        let max = match content_type {
            ContentType::UTF8 => 4 * KB,
            ContentType::Binary => 4 * MB,
        };

        if content_length > max {
            return Err(SBMPError::LargeFrame);
        }

        let Ok(content_length) = usize::try_from(content_length) else {
            return Err(SBMPError::ContentLenDiff);
        };

        Ok(Header::new(version, content_type, content_length))
    }

    pub fn content_type(self) -> ContentType {
        self.content_type
    }

    pub fn content_len(&self) -> usize {
        self.content_length
    }

    pub fn version(&self) -> u8 {
        self.version
    }
}
