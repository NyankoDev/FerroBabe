use rust_asm::error::ClassReadError;
use thiserror::Error;

#[derive(Debug, Error)]
/// Errors that prevent FerroBabe from returning the requested result.
pub enum FerroBabeError {
    /// The byte sequence ended before the eight-byte class-file header was available.
    #[error("class input is too short: expected at least {required} bytes, found {actual}")]
    InputTooShort {
        /// Minimum number of bytes required for the header.
        required: usize,
        /// Number of bytes supplied by the caller.
        actual: usize,
    },

    /// The input header did not contain the Java class-file magic value.
    #[error("input is not a Java class file: found magic 0x{found:08x}")]
    InvalidMagic {
        /// Four-byte value read from the input header.
        found: u32,
    },

    /// The class-file major version is outside FerroBabe's supported range.
    #[error(
        "unsupported Java class-file version {major}.{minor}; FerroBabe supports 45.0 through 70.65535"
    )]
    UnsupportedVersion {
        /// Unsupported class-file major version.
        major: u16,
        /// Class-file minor version paired with `major`.
        minor: u16,
    },

    /// Reading from a caller-provided [`std::io::Read`] source failed.
    #[error("class input could not be read: {source}")]
    InputRead {
        #[source]
        /// Source reader error.
        source: std::io::Error,
    },

    /// The underlying class-file decoder rejected the input.
    #[error("class-file decoding failed: {source}")]
    Decode {
        #[source]
        /// Underlying decoder error.
        source: ClassReadError,
    },

    /// A formatter could not write to its [`std::fmt::Write`] destination.
    #[error("formatted output could not be written")]
    Format {
        #[source]
        /// Formatting destination error.
        source: std::fmt::Error,
    },

    /// Text output was requested for a partial result that has no complete class model.
    #[error("a partial class-file result cannot be formatted")]
    IncompleteDisassembly,
}
