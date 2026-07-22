use rust_asm::nodes::ClassNode;

use super::{ConstantPoolIndex, ConstantRef, Field, Method};
use crate::Diagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The minor and major version numbers stored in a class-file header.
pub struct ClassVersion {
    minor: u16,
    major: u16,
}

impl ClassVersion {
    pub(crate) const fn new(minor: u16, major: u16) -> Self {
        Self { minor, major }
    }

    /// Returns the class-file minor version.
    #[must_use]
    pub fn minor(self) -> u16 {
        self.minor
    }

    /// Returns the class-file major version.
    #[must_use]
    pub fn major(self) -> u16 {
        self.major
    }
}

#[derive(Debug, Clone)]
/// A complete, decoded Java class file.
///
/// This type owns the parsed class-file data. Its member and constant-pool accessors return
/// borrowed views, so no additional model allocation is required while inspecting a class.
pub struct Class {
    node: ClassNode,
}

impl Class {
    pub(crate) fn from_node(node: ClassNode) -> Self {
        Self { node }
    }

    /// Returns the class-file version recorded in the header.
    #[must_use]
    pub fn version(&self) -> ClassVersion {
        ClassVersion {
            minor: self.node.minor_version,
            major: self.node.major_version,
        }
    }

    /// Returns the raw class access-flag bitset.
    ///
    /// Interpret the bits in the context of the Java Virtual Machine Specification's `ClassFile`
    /// access flags.
    #[must_use]
    pub fn access_flags(&self) -> u16 {
        self.node.access_flags
    }

    /// Returns this class's internal JVM name.
    ///
    /// Internal names use slash-separated package segments and are not normalized by FerroBabe.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.node.name
    }

    /// Returns the direct superclass internal name, if the class has one.
    ///
    /// `None` represents the `java/lang/Object` root class.
    #[must_use]
    pub fn super_name(&self) -> Option<&str> {
        self.node.super_name.as_deref()
    }

    /// Returns the `SourceFile` attribute value when present.
    #[must_use]
    pub fn source_file(&self) -> Option<&str> {
        self.node.source_file.as_deref()
    }

    /// Iterates over direct interface internal names in class-file order.
    pub fn interfaces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.node.interfaces.iter().map(String::as_str)
    }

    /// Iterates over fields in class-file order.
    pub fn fields(&self) -> impl ExactSizeIterator<Item = Field<'_>> {
        self.node.fields.iter().map(Field::new)
    }

    /// Iterates over methods in class-file order.
    ///
    /// Each returned method can borrow the class constant pool to resolve exception handler names.
    pub fn methods(&self) -> impl ExactSizeIterator<Item = Method<'_>> {
        self.node
            .methods
            .iter()
            .map(|method| Method::new(method, &self.node.constant_pool))
    }

    /// Iterates over every constant-pool slot, including unusable reserved slots.
    ///
    /// The returned index is the original one-based class-file index. Slot zero is included only
    /// when supplied by the underlying model and is reported as [`ConstantRef::Unusable`].
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

    /// Returns the constant-pool entry at `index`.
    ///
    /// Returns `None` when `index` is outside the decoded constant pool.
    #[must_use]
    pub fn constant(&self, index: ConstantPoolIndex) -> Option<ConstantRef<'_>> {
        self.node
            .constant_pool
            .get(index.get() as usize)
            .map(ConstantRef::from)
    }
}

#[derive(Debug, Clone)]
/// The outcome of parsing one class-file input.
///
/// A complete result contains a [`Class`]. Best-effort recovery returns a partial result when
/// only the class-file header was available; callers can inspect [`Self::version`] and
/// [`Self::diagnostics`] in that case.
pub struct Disassembly {
    header: ClassVersion,
    class: Option<Class>,
    diagnostics: Vec<Diagnostic>,
}

impl Disassembly {
    pub(crate) fn complete(class: Class) -> Self {
        Self {
            header: class.version(),
            class: Some(class),
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn partial(header: ClassVersion, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            header,
            class: None,
            diagnostics,
        }
    }

    /// Returns the class-file version read from the input header.
    ///
    /// This is available for both complete and partial results.
    #[must_use]
    pub fn version(&self) -> ClassVersion {
        self.header
    }

    /// Returns `true` when a complete [`Class`] model is available.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.class.is_some()
    }

    /// Returns the complete parsed class, if decoding finished successfully.
    ///
    /// Returns `None` for a best-effort partial result.
    #[must_use]
    pub fn class(&self) -> Option<&Class> {
        self.class.as_ref()
    }

    /// Returns diagnostics collected during best-effort recovery.
    ///
    /// Complete results currently return an empty slice.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}
