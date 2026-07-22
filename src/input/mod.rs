use crate::FerroBabeError;

pub(crate) const MIN_CLASS_MAJOR_VERSION: u16 = 45;
pub(crate) const MAX_CLASS_MAJOR_VERSION: u16 = 70;
const CLASS_MAGIC: u32 = 0xCAFEBABE;
const CLASS_HEADER_LENGTH: usize = 8;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ClassHeader {
    pub(crate) minor_version: u16,
    pub(crate) major_version: u16,
}

impl ClassHeader {
    pub(crate) fn read(bytes: &[u8]) -> Result<Self, FerroBabeError> {
        if bytes.len() < CLASS_HEADER_LENGTH {
            return Err(FerroBabeError::InputTooShort {
                required: CLASS_HEADER_LENGTH,
                actual: bytes.len(),
            });
        }

        let magic = u32::from_be_bytes(bytes[0..4].try_into().expect("validated length"));
        if magic != CLASS_MAGIC {
            return Err(FerroBabeError::InvalidMagic { found: magic });
        }

        let minor_version = u16::from_be_bytes(bytes[4..6].try_into().expect("validated length"));
        let major_version = u16::from_be_bytes(bytes[6..8].try_into().expect("validated length"));
        if !(MIN_CLASS_MAJOR_VERSION..=MAX_CLASS_MAJOR_VERSION).contains(&major_version) {
            return Err(FerroBabeError::UnsupportedVersion {
                major: major_version,
                minor: minor_version,
            });
        }

        Ok(Self {
            minor_version,
            major_version,
        })
    }
}
