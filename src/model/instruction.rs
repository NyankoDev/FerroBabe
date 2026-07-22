use rust_asm::insn::{Insn, LdcValue, MemberRef};

use super::ConstantPoolIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteOffset(u16);

impl ByteOffset {
    pub(crate) const fn new(offset: u16) -> Self {
        Self(offset)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction<'a> {
    offset: ByteOffset,
    instruction: &'a Insn,
}

impl<'a> Instruction<'a> {
    pub(crate) const fn new(offset: u16, instruction: &'a Insn) -> Self {
        Self {
            offset: ByteOffset::new(offset),
            instruction,
        }
    }

    #[must_use]
    pub const fn offset(&self) -> ByteOffset {
        self.offset
    }

    #[must_use]
    pub fn opcode(&self) -> u8 {
        opcode_of(self.instruction)
    }

    #[must_use]
    pub fn operand(&self) -> InstructionOperand<'a> {
        let offset = i32::from(self.offset.get());

        match self.instruction {
            Insn::Simple(_) => InstructionOperand::None,
            Insn::Int(node) => InstructionOperand::Immediate(node.operand),
            Insn::Var(node) => InstructionOperand::Local(node.var_index),
            Insn::Type(node) => {
                InstructionOperand::ConstantPool(ConstantPoolIndex::new(node.type_index))
            }
            Insn::Field(node) => InstructionOperand::Member(MemberReference::from(&node.field_ref)),
            Insn::Method(node) => {
                InstructionOperand::Member(MemberReference::from(&node.method_ref))
            }
            Insn::InvokeInterface(node) => InstructionOperand::InvokeInterface {
                method: ConstantPoolIndex::new(node.method_index),
                count: node.count,
            },
            Insn::InvokeDynamic(node) => InstructionOperand::InvokeDynamic {
                call_site: ConstantPoolIndex::new(node.method_index),
            },
            Insn::Jump(node) => InstructionOperand::Branch {
                relative: node.offset,
                target: offset + node.offset,
            },
            Insn::Ldc(node) => InstructionOperand::Ldc(LdcValueRef::from(&node.value)),
            Insn::Iinc(node) => InstructionOperand::Increment {
                local: node.var_index,
                amount: node.increment,
            },
            Insn::TableSwitch(node) => InstructionOperand::TableSwitch {
                default: SwitchTarget::new(node.default_offset, offset + node.default_offset),
                low: node.low,
                high: node.high,
                targets: &node.offsets,
                base_offset: self.offset,
            },
            Insn::LookupSwitch(node) => InstructionOperand::LookupSwitch {
                default: SwitchTarget::new(node.default_offset, offset + node.default_offset),
                pairs: &node.pairs,
                base_offset: self.offset,
            },
            Insn::MultiANewArray(node) => InstructionOperand::MultiArray {
                class: ConstantPoolIndex::new(node.type_index),
                dimensions: node.dimensions,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InstructionOperand<'a> {
    None,
    Immediate(i32),
    Local(u16),
    ConstantPool(ConstantPoolIndex),
    Member(MemberReference<'a>),
    InvokeInterface {
        method: ConstantPoolIndex,
        count: u8,
    },
    InvokeDynamic {
        call_site: ConstantPoolIndex,
    },
    Branch {
        relative: i32,
        target: i32,
    },
    Ldc(LdcValueRef<'a>),
    Increment {
        local: u16,
        amount: i16,
    },
    TableSwitch {
        default: SwitchTarget,
        low: i32,
        high: i32,
        targets: &'a [i32],
        base_offset: ByteOffset,
    },
    LookupSwitch {
        default: SwitchTarget,
        pairs: &'a [(i32, i32)],
        base_offset: ByteOffset,
    },
    MultiArray {
        class: ConstantPoolIndex,
        dimensions: u8,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum MemberReference<'a> {
    ConstantPool(ConstantPoolIndex),
    Symbolic {
        owner: &'a str,
        name: &'a str,
        descriptor: &'a str,
    },
}

impl<'a> From<&'a MemberRef> for MemberReference<'a> {
    fn from(value: &'a MemberRef) -> Self {
        match value {
            MemberRef::Index(index) => Self::ConstantPool(ConstantPoolIndex::new(*index)),
            MemberRef::Symbolic {
                owner,
                name,
                descriptor,
            } => Self::Symbolic {
                owner,
                name,
                descriptor,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LdcValueRef<'a> {
    ConstantPool(ConstantPoolIndex),
    String(&'a str),
    TypeDescriptor,
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
}

impl<'a> From<&'a LdcValue> for LdcValueRef<'a> {
    fn from(value: &'a LdcValue) -> Self {
        match value {
            LdcValue::Index(index) => Self::ConstantPool(ConstantPoolIndex::new(*index)),
            LdcValue::String(value) => Self::String(value),
            LdcValue::Type(_) => Self::TypeDescriptor,
            LdcValue::Int(value) => Self::Integer(*value),
            LdcValue::Float(value) => Self::Float(*value),
            LdcValue::Long(value) => Self::Long(*value),
            LdcValue::Double(value) => Self::Double(*value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchTarget {
    relative: i32,
    target: i32,
}

impl SwitchTarget {
    const fn new(relative: i32, target: i32) -> Self {
        Self { relative, target }
    }

    #[must_use]
    pub const fn relative(self) -> i32 {
        self.relative
    }

    #[must_use]
    pub const fn target(self) -> i32 {
        self.target
    }
}

fn opcode_of(instruction: &Insn) -> u8 {
    match instruction {
        Insn::Simple(node) => node.opcode,
        Insn::Int(node) => node.insn.opcode,
        Insn::Var(node) => node.insn.opcode,
        Insn::Type(node) => node.insn.opcode,
        Insn::Field(node) => node.insn.opcode,
        Insn::Method(node) => node.insn.opcode,
        Insn::InvokeInterface(node) => node.insn.opcode,
        Insn::InvokeDynamic(node) => node.insn.opcode,
        Insn::Jump(node) => node.insn.opcode,
        Insn::Ldc(node) => node.insn.opcode,
        Insn::Iinc(node) => node.insn.opcode,
        Insn::TableSwitch(node) => node.insn.opcode,
        Insn::LookupSwitch(node) => node.insn.opcode,
        Insn::MultiANewArray(node) => node.insn.opcode,
    }
}
