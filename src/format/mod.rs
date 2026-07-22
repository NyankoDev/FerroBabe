mod ferro;
mod opcode;

use std::fmt;

use crate::{Class, FerroBabeError};

pub use ferro::FerroFormatter;

pub trait Formatter {
    fn write_class(&self, class: &Class, output: &mut dyn fmt::Write) -> fmt::Result;

    fn format(&self, class: &Class) -> Result<String, FerroBabeError> {
        let mut output = String::new();
        self.write_class(class, &mut output)
            .map_err(|source| FerroBabeError::Format { source })?;
        Ok(output)
    }
}
