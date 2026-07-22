use rust_asm::nodes::ClassNode;

use super::{ConstantPoolIndex, ConstantRef, Field, Method};
use crate::Diagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassVersion {
    minor: u16,
    major: u16,
}

impl ClassVersion {
    #[must_use]
    pub fn minor(self) -> u16 {
        self.minor
    }

    #[must_use]
    pub fn major(self) -> u16 {
        self.major
    }
}

#[derive(Debug, Clone)]
pub struct Class {
    node: ClassNode,
}

impl Class {
    pub(crate) fn from_node(node: ClassNode) -> Self {
        Self { node }
    }

    #[must_use]
    pub fn version(&self) -> ClassVersion {
        ClassVersion {
            minor: self.node.minor_version,
            major: self.node.major_version,
        }
    }

    #[must_use]
    pub fn access_flags(&self) -> u16 {
        self.node.access_flags
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.node.name
    }

    #[must_use]
    pub fn super_name(&self) -> Option<&str> {
        self.node.super_name.as_deref()
    }

    #[must_use]
    pub fn source_file(&self) -> Option<&str> {
        self.node.source_file.as_deref()
    }

    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.node.interfaces.iter().map(String::as_str)
    }

    pub fn fields(&self) -> impl ExactSizeIterator<Item = Field<'_>> {
        self.node.fields.iter().map(Field::new)
    }

    pub fn methods(&self) -> impl ExactSizeIterator<Item = Method<'_>> {
        self.node
            .methods
            .iter()
            .map(|method| Method::new(method, &self.node.constant_pool))
    }

    pub fn constants(&self) -> impl ExactSizeIterator<Item = (ConstantPoolIndex, ConstantRef<'_>)> {
        self.node
            .constant_pool
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                (
                    ConstantPoolIndex::new(index as u16),
                    ConstantRef::from(entry),
                )
            })
    }

    #[must_use]
    pub fn constant(&self, index: ConstantPoolIndex) -> Option<ConstantRef<'_>> {
        self.node
            .constant_pool
            .get(index.get() as usize)
            .map(ConstantRef::from)
    }
}

#[derive(Debug, Clone)]
pub struct Disassembly {
    class: Class,
    diagnostics: Vec<Diagnostic>,
}

impl Disassembly {
    pub(crate) fn new(class: Class, diagnostics: Vec<Diagnostic>) -> Self {
        Self { class, diagnostics }
    }

    #[must_use]
    pub fn class(&self) -> &Class {
        &self.class
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
