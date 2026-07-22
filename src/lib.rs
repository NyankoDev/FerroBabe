mod decode;
mod diagnostic;
mod error;
pub mod format;
mod input;
pub mod model;
mod options;

pub use diagnostic::{Diagnostic, DiagnosticSeverity, DiagnosticStage};
pub use error::FerroBabeError;
pub use format::{FerroFormatter, Formatter};
pub use model::{Class, ClassVersion, Disassembly};
pub use options::{DisassemblerBuilder, DisassemblerOptions, RecoveryMode};

use decode::decode_class;
use input::ClassHeader;

#[derive(Debug, Clone, Default)]
pub struct Disassembler {
    options: DisassemblerOptions,
}

impl Disassembler {
    #[must_use]
    pub fn builder() -> DisassemblerBuilder {
        DisassemblerBuilder::default()
    }

    #[must_use]
    pub fn new(options: DisassemblerOptions) -> Self {
        Self { options }
    }

    #[must_use]
    pub fn options(&self) -> &DisassemblerOptions {
        &self.options
    }

    pub fn parse(&self, bytes: &[u8]) -> Result<Disassembly, FerroBabeError> {
        let header = ClassHeader::read(bytes)?;
        let class = decode_class(bytes, header)?;

        Ok(Disassembly::new(class, Vec::new()))
    }

    pub fn disassemble(&self, bytes: &[u8]) -> Result<String, FerroBabeError> {
        self.disassemble_with(bytes, &FerroFormatter)
    }

    pub fn disassemble_with<F: Formatter>(
        &self,
        bytes: &[u8],
        formatter: &F,
    ) -> Result<String, FerroBabeError> {
        let disassembly = self.parse(bytes)?;
        formatter.format(disassembly.class())
    }
}
