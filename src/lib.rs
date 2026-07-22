#![warn(missing_docs)]
//! FerroBabe disassembles individual Java class files without requiring a Java runtime.
//!
//! [`Disassembler`] is the entry point. It parses bytes or a [`std::io::Read`] source into a
//! [`Disassembly`], which exposes either a complete [`Class`] or the header and diagnostics that
//! were available before decoding stopped. Use [`FerroFormatter`] for FerroBabe's compact text
//! output, or implement [`Formatter`] to render the structured model yourself.

mod decode;
mod diagnostic;
mod error;
/// Text-formatting APIs for parsed class files.
pub mod format;
mod input;
/// Borrowed views over class-file metadata, instructions, and constant-pool entries.
pub mod model;
mod options;

pub use diagnostic::{Diagnostic, DiagnosticSeverity, DiagnosticStage};
pub use error::FerroBabeError;
pub use format::{FerroFormatter, Formatter};
pub use model::{Class, ClassVersion, Disassembly};
pub use options::{DisassemblerBuilder, DisassemblerOptions, RecoveryMode};

use decode::decode_class;
use input::ClassHeader;
use std::io::Read;

#[derive(Debug, Clone, Default)]
/// Parses Java class files into a structured reverse-engineering model.
///
/// The default instance uses [`RecoveryMode::BestEffort`]. It preserves a readable class-file
/// header and records a [`Diagnostic`] when complete decoding cannot continue.
pub struct Disassembler {
    options: DisassemblerOptions,
}

impl Disassembler {
    /// Creates a builder for configuring a disassembler.
    ///
    /// The builder starts with [`RecoveryMode::BestEffort`].
    #[must_use]
    pub fn builder() -> DisassemblerBuilder {
        DisassemblerBuilder::default()
    }

    /// Creates a disassembler with the supplied options.
    #[must_use]
    pub fn new(options: DisassemblerOptions) -> Self {
        Self { options }
    }

    /// Returns the options used by this disassembler.
    #[must_use]
    pub fn options(&self) -> &DisassemblerOptions {
        &self.options
    }

    /// Parses one complete Java class-file byte sequence.
    ///
    /// In best-effort mode, an error after the class-file header is read returns a partial
    /// [`Disassembly`] with diagnostics. In strict mode, the same decoding failure is returned as
    /// [`FerroBabeError`]. Invalid, incomplete, or unsupported headers always return an error.
    ///
    /// # Errors
    ///
    /// Returns an error when the input has no readable supported class-file header, or when
    /// strict recovery is selected and decoding fails.
    pub fn parse(&self, bytes: &[u8]) -> Result<Disassembly, FerroBabeError> {
        let header = ClassHeader::read(bytes)?;
        match decode_class(bytes, header) {
            Ok(class) => Ok(Disassembly::complete(class)),
            Err(error) if self.options.recovery() == RecoveryMode::BestEffort => Ok(
                Disassembly::partial(header.into(), vec![Diagnostic::from_decode_error(&error)]),
            ),
            Err(error) => Err(error),
        }
    }

    /// Reads and parses one Java class file from `reader`.
    ///
    /// The complete reader is buffered before parsing, so the returned [`Disassembly`] does not
    /// borrow from `reader`.
    ///
    /// # Errors
    ///
    /// Returns [`FerroBabeError::InputRead`] when reading fails, or any error documented by
    /// [`Self::parse`] after the bytes have been read.
    pub fn parse_reader<R: Read>(&self, mut reader: R) -> Result<Disassembly, FerroBabeError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .map_err(|source| FerroBabeError::InputRead { source })?;
        self.parse(&bytes)
    }

    /// Parses `bytes` and renders a complete class with [`FerroFormatter`].
    ///
    /// # Errors
    ///
    /// Returns parse errors, [`FerroBabeError::IncompleteDisassembly`] for a recovered partial
    /// result, or a formatting error from the default formatter.
    pub fn disassemble(&self, bytes: &[u8]) -> Result<String, FerroBabeError> {
        self.disassemble_with(bytes, &FerroFormatter)
    }

    /// Parses `bytes` and renders the resulting complete class with `formatter`.
    ///
    /// `formatter` receives the same [`Class`] view available through [`Disassembly::class`].
    ///
    /// # Errors
    ///
    /// Returns parse errors, [`FerroBabeError::IncompleteDisassembly`] for a partial result, or
    /// [`FerroBabeError::Format`] when `formatter` cannot write its output.
    pub fn disassemble_with<F: Formatter>(
        &self,
        bytes: &[u8],
        formatter: &F,
    ) -> Result<String, FerroBabeError> {
        let disassembly = self.parse(bytes)?;
        let class = disassembly
            .class()
            .ok_or(FerroBabeError::IncompleteDisassembly)?;
        formatter.format(class)
    }
}
