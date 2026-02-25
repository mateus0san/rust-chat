use crate::sbmp::SBMPError;

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum ContentType {
    UTF8,
    Binary,
}

impl TryFrom<u8> for ContentType {
    type Error = SBMPError;

    fn try_from(content_type: u8) -> Result<Self, Self::Error> {
        let content_type = match content_type {
            0 => ContentType::UTF8,
            1 => ContentType::Binary,
            _ => return Err(SBMPError::UknownContentType(content_type)),
        };

        Ok(content_type)
    }
}
