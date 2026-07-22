use rust_asm::class_reader::{AttributeInfo, ExceptionTableEntry};
use rust_asm::constant_pool::CpInfo;
use rust_asm::nodes::{FieldNode, MethodNode};

use super::{ByteOffset, Instruction, StackMapFrame};

#[derive(Debug, Clone, Copy)]
/// A borrowed view of a field declaration.
pub struct Field<'a> {
    inner: &'a FieldNode,
}

impl<'a> Field<'a> {
    pub(crate) const fn new(inner: &'a FieldNode) -> Self {
        Self { inner }
    }

    /// Returns the raw field access-flag bitset.
    #[must_use]
    pub fn access_flags(&self) -> u16 {
        self.inner.access_flags
    }

    /// Returns the field name exactly as stored in the class file.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Returns the JVM field descriptor.
    #[must_use]
    pub fn descriptor(&self) -> &str {
        &self.inner.descriptor
    }
}

#[derive(Debug, Clone, Copy)]
/// A borrowed view of a method declaration and its optional code.
pub struct Method<'a> {
    inner: &'a MethodNode,
    constant_pool: &'a [CpInfo],
}

impl<'a> Method<'a> {
    pub(crate) const fn new(inner: &'a MethodNode, constant_pool: &'a [CpInfo]) -> Self {
        Self {
            inner,
            constant_pool,
        }
    }

    /// Returns the raw method access-flag bitset.
    #[must_use]
    pub fn access_flags(&self) -> u16 {
        self.inner.access_flags
    }

    /// Returns the method name exactly as stored in the class file.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Returns the JVM method descriptor.
    #[must_use]
    pub fn descriptor(&self) -> &str {
        &self.inner.descriptor
    }

    /// Returns whether this method has a `Code` attribute.
    ///
    /// Abstract and native methods normally return `false`.
    #[must_use]
    pub fn has_code(&self) -> bool {
        self.inner.has_code
    }

    /// Returns the `Code` attribute's declared maximum operand-stack depth.
    ///
    /// Returns zero for methods without code.
    #[must_use]
    pub fn max_stack(&self) -> u16 {
        self.inner.max_stack
    }

    /// Returns the `Code` attribute's declared maximum local-variable count.
    ///
    /// Returns zero for methods without code.
    #[must_use]
    pub fn max_locals(&self) -> u16 {
        self.inner.max_locals
    }

    /// Iterates over bytecode instructions in original order when code is present.
    ///
    /// Returns `None` for methods without a `Code` attribute. Each instruction retains its
    /// original bytecode offset and borrows from this method.
    pub fn instructions(&self) -> Option<impl ExactSizeIterator<Item = Instruction<'a>> + '_> {
        self.inner.has_code.then(|| {
            self.inner
                .instructions
                .insns()
                .iter()
                .zip(self.inner.instruction_offsets.iter().copied())
                .map(|(instruction, offset)| Instruction::new(offset, instruction))
        })
    }

    /// Iterates over exception-table entries in class-file order.
    ///
    /// Methods without code return an empty iterator.
    pub fn exception_handlers(&self) -> impl ExactSizeIterator<Item = ExceptionHandler<'a>> + '_ {
        self.inner
            .exception_table
            .iter()
            .map(|entry| ExceptionHandler::new(entry, self.constant_pool))
    }

    /// Iterates over `StackMapTable` frames when the method has that attribute.
    ///
    /// The class-file format stores frames as deltas from preceding frames. The
    /// returned views preserve that encoding and resolve object verification
    /// types against the method's constant pool.
    pub fn stack_map_frames(
        &self,
    ) -> Option<impl ExactSizeIterator<Item = StackMapFrame<'a>> + '_> {
        let entries = self
            .inner
            .code_attributes
            .iter()
            .find_map(|attribute| match attribute {
                AttributeInfo::StackMapTable { entries } => Some(entries),
                _ => None,
            })?;

        Some(
            entries
                .iter()
                .map(|frame| StackMapFrame::new(frame, self.constant_pool)),
        )
    }
}

#[derive(Debug, Clone, Copy)]
/// One protected bytecode range and its exception handler.
pub struct ExceptionHandler<'a> {
    start: ByteOffset,
    end: ByteOffset,
    handler: ByteOffset,
    catch_type: Option<&'a str>,
}

impl<'a> ExceptionHandler<'a> {
    fn new(entry: &'a ExceptionTableEntry, constant_pool: &'a [CpInfo]) -> Self {
        let catch_type = (entry.catch_type != 0)
            .then(|| class_name(constant_pool, entry.catch_type))
            .flatten();

        Self {
            start: ByteOffset::new(entry.start_pc),
            end: ByteOffset::new(entry.end_pc),
            handler: ByteOffset::new(entry.handler_pc),
            catch_type,
        }
    }

    /// Returns the inclusive start offset of the protected range.
    #[must_use]
    pub const fn start(&self) -> ByteOffset {
        self.start
    }

    /// Returns the exclusive end offset of the protected range.
    #[must_use]
    pub const fn end(&self) -> ByteOffset {
        self.end
    }

    /// Returns the bytecode offset where the handler begins.
    #[must_use]
    pub const fn handler(&self) -> ByteOffset {
        self.handler
    }

    /// Returns the caught exception's internal class name.
    ///
    /// Returns `None` for a catch-all handler or when the referenced class could not be resolved.
    #[must_use]
    pub const fn catch_type(&self) -> Option<&'a str> {
        self.catch_type
    }
}

fn class_name(constant_pool: &[CpInfo], index: u16) -> Option<&str> {
    let CpInfo::Class { name_index } = constant_pool.get(index as usize)? else {
        return None;
    };
    let CpInfo::Utf8(name) = constant_pool.get(*name_index as usize)? else {
        return None;
    };
    Some(name)
}
