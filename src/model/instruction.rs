use rust_asm::insn::{Insn, LdcValue, MemberRef};

use super::ConstantPoolIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A byte offset within a method's `Code` attribute.
pub struct ByteOffset(u16);

impl ByteOffset {
    pub(crate) const fn new(offset: u16) -> Self {
        Self(offset)
    }

    /// Returns the raw byte offset.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
/// A borrowed JVM instruction paired with its original bytecode offset.
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

    /// Returns the instruction's offset within its containing method's code array.
    #[must_use]
    pub const fn offset(&self) -> ByteOffset {
        self.offset
    }

    /// Returns the raw JVM opcode byte.
    #[must_use]
    pub fn opcode(&self) -> u8 {
        opcode_of(self.instruction)
    }

    /// Returns this instruction's decoded operand while preserving raw indices and offsets.
    ///
    /// Branch targets are calculated relative to [`Self::offset`]. Switch target slices contain
    /// relative offsets; their associated `base_offset` identifies the instruction from which to
    /// calculate an absolute target.
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
/// A decoded JVM instruction operand.
///
/// Variants expose the operand representation used by the class file. In particular, indices are
/// not rewritten into source-level names and switch targets retain their relative offsets.
pub enum InstructionOperand<'a> {
    /// An instruction with no explicit operand.
    None,
    /// A signed immediate operand.
    Immediate(i32),
    /// A local-variable slot index.
    Local(u16),
    /// A direct constant-pool index.
    ConstantPool(ConstantPoolIndex),
    /// A field or method reference.
    Member(MemberReference<'a>),
    /// An `invokeinterface` method index and its encoded argument count.
    InvokeInterface {
        /// Index of the interface method reference.
        method: ConstantPoolIndex,
        /// Encoded argument count byte.
        count: u8,
    },
    /// An `invokedynamic` call-site index.
    InvokeDynamic {
        /// Index of the invokedynamic constant-pool entry.
        call_site: ConstantPoolIndex,
    },
    /// A branch displacement and its calculated target offset.
    Branch {
        /// Signed displacement encoded by the instruction.
        relative: i32,
        /// Offset obtained by adding `relative` to the instruction offset.
        target: i32,
    },
    /// A value loaded by an `ldc`, `ldc_w`, or `ldc2_w` instruction.
    Ldc(LdcValueRef<'a>),
    /// A local-variable increment.
    Increment {
        /// Local-variable slot index.
        local: u16,
        /// Signed increment amount.
        amount: i16,
    },
    /// A `tableswitch` operand.
    TableSwitch {
        /// Default target displacement and absolute target.
        default: SwitchTarget,
        /// Lowest matching key.
        low: i32,
        /// Highest matching key.
        high: i32,
        /// Relative target offsets in key order.
        targets: &'a [i32],
        /// Offset of the `tableswitch` instruction.
        base_offset: ByteOffset,
    },
    /// A `lookupswitch` operand.
    LookupSwitch {
        /// Default target displacement and absolute target.
        default: SwitchTarget,
        /// Match-key and relative-target pairs in class-file order.
        pairs: &'a [(i32, i32)],
        /// Offset of the `lookupswitch` instruction.
        base_offset: ByteOffset,
    },
    /// A `multianewarray` class reference and dimension count.
    MultiArray {
        /// Index of the array class entry.
        class: ConstantPoolIndex,
        /// Number of dimensions to allocate.
        dimensions: u8,
    },
}

#[derive(Debug, Clone, Copy)]
/// A field or method reference used by an instruction.
pub enum MemberReference<'a> {
    /// A reference preserved as a constant-pool index.
    ConstantPool(ConstantPoolIndex),
    /// A reference already resolved into its JVM symbolic components.
    Symbolic {
        /// Declaring class or interface internal name.
        owner: &'a str,
        /// Member name.
        name: &'a str,
        /// JVM field or method descriptor.
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
/// A value loaded by an LDC-family instruction.
pub enum LdcValueRef<'a> {
    /// A constant-pool entry reference.
    ConstantPool(ConstantPoolIndex),
    /// A resolved string literal.
    String(&'a str),
    /// A type literal whose exact descriptor is not retained by this view.
    TypeDescriptor,
    /// A 32-bit integer literal.
    Integer(i32),
    /// A 32-bit floating-point literal.
    Float(f32),
    /// A 64-bit integer literal.
    Long(i64),
    /// A 64-bit floating-point literal.
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
/// A switch or branch target represented as both relative and absolute offsets.
pub struct SwitchTarget {
    relative: i32,
    target: i32,
}

impl SwitchTarget {
    const fn new(relative: i32, target: i32) -> Self {
        Self { relative, target }
    }

    /// Returns the signed displacement encoded in the class file.
    #[must_use]
    pub const fn relative(self) -> i32 {
        self.relative
    }

    /// Returns the target offset calculated from the enclosing switch instruction.
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
