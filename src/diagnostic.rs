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
