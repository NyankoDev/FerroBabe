#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RecoveryMode {
    #[default]
    BestEffort,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisassemblerOptions {
    recovery: RecoveryMode,
}

impl Default for DisassemblerOptions {
    fn default() -> Self {
        Self {
            recovery: RecoveryMode::BestEffort,
        }
    }
}

impl DisassemblerOptions {
    #[must_use]
    pub fn recovery(&self) -> RecoveryMode {
        self.recovery
    }
}

#[derive(Debug, Clone, Default)]
pub struct DisassemblerBuilder {
    options: DisassemblerOptions,
}

impl DisassemblerBuilder {
    #[must_use]
    pub fn recovery(mut self, recovery: RecoveryMode) -> Self {
        self.options.recovery = recovery;
        self
    }

    #[must_use]
    pub fn build(self) -> Disassembler {
        Disassembler::new(self.options)
    }
}

use crate::Disassembler;
