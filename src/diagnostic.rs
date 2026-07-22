#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticStage {
    Header,
    ConstantPool,
    Member,
    Attribute,
    Code,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    #[must_use]
    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    #[must_use]
    pub fn stage(&self) -> DiagnosticStage {
        self.stage
    }

    #[must_use]
    pub fn offset(&self) -> Option<usize> {
        self.offset
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
use rust_asm::error::ClassReadError;

use crate::FerroBabeError;
