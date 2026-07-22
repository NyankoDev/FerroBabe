use rust_asm::error::ClassReadError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FerroBabeError {
    #[error("class input is too short: expected at least {required} bytes, found {actual}")]
    InputTooShort { required: usize, actual: usize },

    #[error("input is not a Java class file: found magic 0x{found:08x}")]
    InvalidMagic { found: u32 },

    #[error(
        "unsupported Java class-file version {major}.{minor}; FerroBabe supports 45.0 through 70.65535"
    )]
    UnsupportedVersion { major: u16, minor: u16 },

    #[error("class-file decoding failed: {source}")]
    Decode {
        #[source]
        source: ClassReadError,
    },

    #[error("formatted output could not be written")]
    Format {
        #[source]
        source: std::fmt::Error,
    },
}
