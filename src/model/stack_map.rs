use rust_asm::class_reader::{
    StackMapFrame as RawStackMapFrame, VerificationTypeInfo as RawVerificationType,
};
use rust_asm::constant_pool::CpInfo;

use super::ConstantPoolIndex;

/// A borrowed view of one `StackMapTable` frame.
///
/// Frame offsets are encoded as deltas by the class-file format. Use
/// [`Self::offset_delta`] together with preceding frames to recover an absolute
/// bytecode offset.
#[derive(Debug, Clone, Copy)]
pub struct StackMapFrame<'a> {
    inner: &'a RawStackMapFrame,
    constant_pool: &'a [CpInfo],
}

impl<'a> StackMapFrame<'a> {
    pub(crate) const fn new(inner: &'a RawStackMapFrame, constant_pool: &'a [CpInfo]) -> Self {
        Self {
            inner,
            constant_pool,
        }
    }

    /// Returns this frame's encoded offset delta.
    #[must_use]
    pub const fn offset_delta(&self) -> u16 {
        match self.inner {
            RawStackMapFrame::SameFrame { offset_delta }
            | RawStackMapFrame::SameLocals1StackItemFrame { offset_delta, .. }
            | RawStackMapFrame::SameLocals1StackItemFrameExtended { offset_delta, .. }
            | RawStackMapFrame::ChopFrame { offset_delta, .. }
            | RawStackMapFrame::SameFrameExtended { offset_delta }
            | RawStackMapFrame::AppendFrame { offset_delta, .. }
            | RawStackMapFrame::FullFrame { offset_delta, .. } => *offset_delta,
        }
    }

    /// Returns the frame encoding kind.
    #[must_use]
    pub const fn kind(&self) -> StackMapFrameKind {
        match self.inner {
            RawStackMapFrame::SameFrame { .. } => StackMapFrameKind::Same,
            RawStackMapFrame::SameLocals1StackItemFrame { .. }
            | RawStackMapFrame::SameLocals1StackItemFrameExtended { .. } => {
                StackMapFrameKind::SameLocalsOneStackItem
            }
            RawStackMapFrame::ChopFrame { k, .. } => StackMapFrameKind::Chop { count: *k },
            RawStackMapFrame::SameFrameExtended { .. } => StackMapFrameKind::SameExtended,
            RawStackMapFrame::AppendFrame { .. } => StackMapFrameKind::Append,
            RawStackMapFrame::FullFrame { .. } => StackMapFrameKind::Full,
        }
    }

    /// Returns the local entries encoded by an append or full frame.
    ///
    /// Returns `None` for same and chop frames, whose locals are inherited from
    /// the preceding frame.
    #[must_use]
    pub fn locals(&self) -> Option<VerificationTypes<'a>> {
        let values = match self.inner {
            RawStackMapFrame::AppendFrame { locals, .. }
            | RawStackMapFrame::FullFrame { locals, .. } => locals,
            _ => return None,
        };
        Some(VerificationTypes::new(values, self.constant_pool))
    }

    /// Returns the stack entries encoded by this frame.
    ///
    /// Same and chop frames have an empty operand stack, so they return `None`.
    #[must_use]
    pub fn stack(&self) -> Option<VerificationTypes<'a>> {
        match self.inner {
            RawStackMapFrame::SameLocals1StackItemFrame { stack, .. }
            | RawStackMapFrame::SameLocals1StackItemFrameExtended { stack, .. } => {
                Some(VerificationTypes::single(stack, self.constant_pool))
            }
            RawStackMapFrame::FullFrame { stack, .. } => {
                Some(VerificationTypes::new(stack, self.constant_pool))
            }
            _ => None,
        }
    }
}

/// Encoding form of a [`StackMapFrame`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackMapFrameKind {
    /// A compact frame with inherited locals and an empty operand stack.
    Same,
    /// A frame with inherited locals and one operand-stack entry.
    SameLocalsOneStackItem,
    /// A frame that removes local entries from the preceding frame.
    Chop {
        /// Number of local entries removed from the preceding frame.
        count: u8,
    },
    /// An extended same frame with a wider offset delta.
    SameExtended,
    /// A frame that appends local entries to the preceding frame.
    Append,
    /// A frame that supplies complete local and operand-stack entries.
    Full,
}

/// A borrowed sequence of verification types in a stack-map frame.
#[derive(Debug, Clone, Copy)]
pub struct VerificationTypes<'a> {
    values: &'a [RawVerificationType],
    constant_pool: &'a [CpInfo],
}

impl<'a> VerificationTypes<'a> {
    const fn new(values: &'a [RawVerificationType], constant_pool: &'a [CpInfo]) -> Self {
        Self {
            values,
            constant_pool,
        }
    }

    const fn single(value: &'a RawVerificationType, constant_pool: &'a [CpInfo]) -> Self {
        Self::new(std::slice::from_ref(value), constant_pool)
    }

    /// Returns the number of verification-type entries.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns whether this sequence has no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterates over verification types in class-file order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = VerificationType<'a>> + '_ {
        self.values
            .iter()
            .map(|value| VerificationType::from_raw(value, self.constant_pool))
    }
}

/// A verification type stored in a `StackMapTable` frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationType<'a> {
    /// An unusable local-variable slot.
    Top,
    /// A category-one integral value.
    Integer,
    /// A category-one floating-point value.
    Float,
    /// A category-two integral value.
    Long,
    /// A category-two floating-point value.
    Double,
    /// The JVM `null` value.
    Null,
    /// The receiver in an instance constructor before initialization completes.
    UninitializedThis,
    /// A reference to a class or array type.
    Object {
        /// Resolved internal class name or array descriptor, when available.
        internal_name: Option<&'a str>,
        /// Raw constant-pool index of the class entry.
        constant_pool_index: ConstantPoolIndex,
    },
    /// An object allocated by `new` before constructor initialization completes.
    Uninitialized {
        /// Bytecode offset of the allocating `new` instruction.
        offset: u16,
    },
}

impl<'a> VerificationType<'a> {
    fn from_raw(value: &'a RawVerificationType, constant_pool: &'a [CpInfo]) -> Self {
        match value {
            RawVerificationType::Top => Self::Top,
            RawVerificationType::Integer => Self::Integer,
            RawVerificationType::Float => Self::Float,
            RawVerificationType::Long => Self::Long,
            RawVerificationType::Double => Self::Double,
            RawVerificationType::Null => Self::Null,
            RawVerificationType::UninitializedThis => Self::UninitializedThis,
            RawVerificationType::Object { cpool_index } => Self::Object {
                internal_name: class_name(constant_pool, *cpool_index),
                constant_pool_index: ConstantPoolIndex::new(*cpool_index),
            },
            RawVerificationType::Uninitialized { offset } => {
                Self::Uninitialized { offset: *offset }
            }
        }
    }
}

fn class_name(constant_pool: &[CpInfo], index: u16) -> Option<&str> {
    let CpInfo::Class { name_index } = constant_pool.get(usize::from(index))? else {
        return None;
    };
    let CpInfo::Utf8(name) = constant_pool.get(usize::from(*name_index))? else {
        return None;
    };
    Some(name)
}
