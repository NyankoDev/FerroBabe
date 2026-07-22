#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Indicates whether a recoverable parse finding is informational or erroneous.
pub enum DiagnosticSeverity {
    /// A non-fatal finding that does not invalidate the returned information.
    Warning,
    /// A decoding failure represented in a best-effort partial result.
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Identifies the class-file area associated with a [`Diagnostic`].
pub enum DiagnosticStage {
    /// The class-file header.
    Header,
    /// The constant-pool table or an index into it.
    ConstantPool,
    /// A field, method, or another member structure.
    Member,
    /// A class, member, or code attribute.
    Attribute,
    /// A method's bytecode instruction stream.
    Code,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A recoverable parsing finding returned with a partial [`crate::Disassembly`].
pub struct Diagnostic {
    severity: DiagnosticSeverity,
    stage: DiagnosticStage,
    offset: Option<usize>,
    message: String,
}

impl Diagnostic {
    pub(crate) fn from_decode_error(error: &FerroBabeError) -> Self {
        let (stage, offset, message) = match error {
            FerroBabeError::Decode { source } => match source {
                ClassReadError::UnexpectedEof => {
                    (DiagnosticStage::Member, None, source.to_string())
                }
                ClassReadError::InvalidConstantPoolTag(_) | ClassReadError::InvalidIndex(_) => {
                    (DiagnosticStage::ConstantPool, None, source.to_string())
                }
                ClassReadError::InvalidAttribute(_) => {
                    (DiagnosticStage::Attribute, None, source.to_string())
                }
                ClassReadError::InvalidOpcode { offset, .. } => {
                    (DiagnosticStage::Code, Some(*offset), source.to_string())
                }
                ClassReadError::InvalidMagic(_) | ClassReadError::InvalidClassVersion(_) => {
                    (DiagnosticStage::Header, None, source.to_string())
                }
                ClassReadError::Utf8Error(_) => {
                    (DiagnosticStage::ConstantPool, None, source.to_string())
                }
            },
            _ => (DiagnosticStage::Header, None, error.to_string()),
        };

        Self {
            severity: DiagnosticSeverity::Error,
            stage,
            offset,
            message,
        }
    }

    /// Returns the severity assigned to this finding.
    #[must_use]
    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the class-file area where this finding occurred.
    #[must_use]
    pub fn stage(&self) -> DiagnosticStage {
        self.stage
    }

    /// Returns the reported byte offset when the decoder supplied one.
    ///
    /// Offsets for bytecode findings are relative to the method's code array.
    #[must_use]
    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    /// Returns the human-readable decoder message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
use rust_asm::error::ClassReadError;

use crate::FerroBabeError;
