#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// Selects how the disassembler handles a decoding failure after the header.
pub enum RecoveryMode {
    #[default]
    /// Return the readable header and a diagnostic instead of discarding all parse information.
    BestEffort,
    /// Return the decoding failure as an error.
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Immutable configuration used to create a [`Disassembler`](crate::Disassembler).
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
    /// Returns the configured recovery mode.
    #[must_use]
    pub fn recovery(&self) -> RecoveryMode {
        self.recovery
    }
}

#[derive(Debug, Clone, Default)]
/// Configures a [`Disassembler`](crate::Disassembler) before construction.
pub struct DisassemblerBuilder {
    options: DisassemblerOptions,
}

impl DisassemblerBuilder {
    /// Sets the recovery mode used by the resulting disassembler.
    ///
    /// The supplied mode replaces the builder's previous recovery mode.
    #[must_use]
    pub fn recovery(mut self, recovery: RecoveryMode) -> Self {
        self.options.recovery = recovery;
        self
    }

    /// Builds a [`Disassembler`](crate::Disassembler) from these options.
    #[must_use]
    pub fn build(self) -> Disassembler {
        Disassembler::new(self.options)
    }
}

use crate::Disassembler;
