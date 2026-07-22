use rust_asm::class_reader::ExceptionTableEntry;
use rust_asm::constant_pool::CpInfo;
use rust_asm::nodes::{FieldNode, MethodNode};

use super::{ByteOffset, Instruction};

#[derive(Debug, Clone, Copy)]
pub struct Field<'a> {
    inner: &'a FieldNode,
}

impl<'a> Field<'a> {
    pub(crate) const fn new(inner: &'a FieldNode) -> Self {
        Self { inner }
    }

    #[must_use]
    pub fn access_flags(&self) -> u16 {
        self.inner.access_flags
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    #[must_use]
    pub fn descriptor(&self) -> &str {
        &self.inner.descriptor
    }
}

#[derive(Debug, Clone, Copy)]
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

    #[must_use]
    pub fn access_flags(&self) -> u16 {
        self.inner.access_flags
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    #[must_use]
    pub fn descriptor(&self) -> &str {
        &self.inner.descriptor
    }

    #[must_use]
    pub fn has_code(&self) -> bool {
        self.inner.has_code
    }

    #[must_use]
    pub fn max_stack(&self) -> u16 {
        self.inner.max_stack
    }

    #[must_use]
    pub fn max_locals(&self) -> u16 {
        self.inner.max_locals
    }

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

    pub fn exception_handlers(&self) -> impl ExactSizeIterator<Item = ExceptionHandler<'a>> + '_ {
        self.inner
            .exception_table
            .iter()
            .map(|entry| ExceptionHandler::new(entry, self.constant_pool))
    }
}

#[derive(Debug, Clone, Copy)]
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

    #[must_use]
    pub const fn start(&self) -> ByteOffset {
        self.start
    }

    #[must_use]
    pub const fn end(&self) -> ByteOffset {
        self.end
    }

    #[must_use]
    pub const fn handler(&self) -> ByteOffset {
        self.handler
    }

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
