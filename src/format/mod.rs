//! Text-formatting APIs for complete [`Class`](crate::Class) models.
//!
//! A [`Formatter`] may stream to any [`std::fmt::Write`] destination. Its provided
//! [`Formatter::format`] method collects that output into a [`String`].

mod ferro;
mod opcode;

use std::fmt;

use crate::{Class, FerroBabeError};

pub use ferro::FerroFormatter;

/// Renders a complete class model into text.
///
/// Implement this trait when a consumer needs a presentation other than the compact
/// reverse-engineering output produced by [`FerroFormatter`]. Implementations receive only a
/// complete [`Class`]; partial disassemblies must be handled by the caller.
pub trait Formatter {
    /// Writes `class` to `output`.
    ///
    /// # Errors
    ///
    /// Returns [`std::fmt::Error`] when `output` rejects a write.
    fn write_class(&self, class: &Class, output: &mut dyn fmt::Write) -> fmt::Result;

    /// Renders `class` into a newly allocated string.
    ///
    /// # Errors
    ///
    /// Returns [`FerroBabeError::Format`] when [`Self::write_class`] cannot write to the string
    /// destination.
    fn format(&self, class: &Class) -> Result<String, FerroBabeError> {
        let mut output = String::new();
        self.write_class(class, &mut output)
            .map_err(|source| FerroBabeError::Format { source })?;
        Ok(output)
    }
}
