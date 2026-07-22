//! Borrowed views over data decoded from a Java class file.
//!
//! The types in this module expose immutable information owned by a [`Class`]. Iterators and
//! references returned from a class, field, or method are valid only while that owner is alive.

mod class;
mod constant;
mod instruction;
mod member;
mod stack_map;

pub use class::{Class, ClassVersion, Disassembly};
pub use constant::{ConstantPoolIndex, ConstantRef};
pub use instruction::{
    ByteOffset, Instruction, InstructionOperand, LdcValueRef, MemberReference, SwitchTarget,
};
pub use member::{ExceptionHandler, Field, Method};
pub use stack_map::{StackMapFrame, StackMapFrameKind, VerificationType, VerificationTypes};
