use rust_asm::constant_pool::CpInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// An index into a class file's constant-pool table.
///
/// Index values are preserved exactly as stored in the class file. They are one-based in valid
/// class files, and some values can refer to unusable reserved slots after `Long` or `Double`
/// constants.
pub struct ConstantPoolIndex(u16);

impl ConstantPoolIndex {
    pub(crate) const fn new(index: u16) -> Self {
        Self(index)
    }

    /// Returns the raw constant-pool index value.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
/// A borrowed view of one constant-pool entry.
///
/// Index-bearing variants retain their raw class-file indices instead of resolving or rewriting
/// the referenced entries.
pub enum ConstantRef<'a> {
    /// A reserved constant-pool slot with no value.
    Unusable,
    /// A modified UTF-8 string decoded by the underlying parser.
    Utf8(&'a str),
    /// A 32-bit integer literal.
    Integer(i32),
    /// A 32-bit floating-point literal.
    Float(f32),
    /// A 64-bit integer literal.
    Long(i64),
    /// A 64-bit floating-point literal.
    Double(f64),
    /// A class reference whose name is stored in another constant-pool entry.
    Class {
        /// Index of the UTF-8 internal class name.
        name: ConstantPoolIndex,
    },
    /// A string constant whose value is stored in another constant-pool entry.
    String {
        /// Index of the UTF-8 string value.
        value: ConstantPoolIndex,
    },
    /// A field member reference.
    FieldReference {
        /// Index of the declaring class entry.
        class: ConstantPoolIndex,
        /// Index of the member name-and-descriptor entry.
        name_and_type: ConstantPoolIndex,
    },
    /// A class method reference.
    MethodReference {
        /// Index of the declaring class entry.
        class: ConstantPoolIndex,
        /// Index of the member name-and-descriptor entry.
        name_and_type: ConstantPoolIndex,
    },
    /// An interface method reference.
    InterfaceMethodReference {
        /// Index of the declaring interface entry.
        class: ConstantPoolIndex,
        /// Index of the member name-and-descriptor entry.
        name_and_type: ConstantPoolIndex,
    },
    /// A member name and descriptor pair.
    NameAndType {
        /// Index of the UTF-8 member name.
        name: ConstantPoolIndex,
        /// Index of the UTF-8 descriptor.
        descriptor: ConstantPoolIndex,
    },
    /// A method-handle constant.
    MethodHandle {
        /// Raw JVM reference-kind value.
        reference_kind: u8,
        /// Index of the referenced field or method entry.
        reference: ConstantPoolIndex,
    },
    /// A method-type constant.
    MethodType {
        /// Index of the UTF-8 method descriptor.
        descriptor: ConstantPoolIndex,
    },
    /// A dynamically computed constant.
    Dynamic {
        /// Index into the `BootstrapMethods` attribute.
        bootstrap_method: u16,
        /// Index of the name-and-descriptor entry.
        name_and_type: ConstantPoolIndex,
    },
    /// An `invokedynamic` call-site constant.
    InvokeDynamic {
        /// Index into the `BootstrapMethods` attribute.
        bootstrap_method: u16,
        /// Index of the name-and-descriptor entry.
        name_and_type: ConstantPoolIndex,
    },
    /// A Java Platform Module System module constant.
    Module {
        /// Index of the UTF-8 module name.
        name: ConstantPoolIndex,
    },
    /// A Java Platform Module System package constant.
    Package {
        /// Index of the UTF-8 package name.
        name: ConstantPoolIndex,
    },
}

impl<'a> From<&'a CpInfo> for ConstantRef<'a> {
    fn from(value: &'a CpInfo) -> Self {
        match value {
            CpInfo::Unusable => Self::Unusable,
            CpInfo::Utf8(value) => Self::Utf8(value),
            CpInfo::Integer(value) => Self::Integer(*value),
            CpInfo::Float(value) => Self::Float(*value),
            CpInfo::Long(value) => Self::Long(*value),
            CpInfo::Double(value) => Self::Double(*value),
            CpInfo::Class { name_index } => Self::Class {
                name: ConstantPoolIndex::new(*name_index),
            },
            CpInfo::String { string_index } => Self::String {
                value: ConstantPoolIndex::new(*string_index),
            },
            CpInfo::Fieldref {
                class_index,
                name_and_type_index,
            } => Self::FieldReference {
                class: ConstantPoolIndex::new(*class_index),
                name_and_type: ConstantPoolIndex::new(*name_and_type_index),
            },
            CpInfo::Methodref {
                class_index,
                name_and_type_index,
            } => Self::MethodReference {
                class: ConstantPoolIndex::new(*class_index),
                name_and_type: ConstantPoolIndex::new(*name_and_type_index),
            },
            CpInfo::InterfaceMethodref {
                class_index,
                name_and_type_index,
            } => Self::InterfaceMethodReference {
                class: ConstantPoolIndex::new(*class_index),
                name_and_type: ConstantPoolIndex::new(*name_and_type_index),
            },
            CpInfo::NameAndType {
                name_index,
                descriptor_index,
            } => Self::NameAndType {
                name: ConstantPoolIndex::new(*name_index),
                descriptor: ConstantPoolIndex::new(*descriptor_index),
            },
            CpInfo::MethodHandle {
                reference_kind,
                reference_index,
            } => Self::MethodHandle {
                reference_kind: *reference_kind,
                reference: ConstantPoolIndex::new(*reference_index),
            },
            CpInfo::MethodType { descriptor_index } => Self::MethodType {
                descriptor: ConstantPoolIndex::new(*descriptor_index),
            },
            CpInfo::Dynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => Self::Dynamic {
                bootstrap_method: *bootstrap_method_attr_index,
                name_and_type: ConstantPoolIndex::new(*name_and_type_index),
            },
            CpInfo::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => Self::InvokeDynamic {
                bootstrap_method: *bootstrap_method_attr_index,
                name_and_type: ConstantPoolIndex::new(*name_and_type_index),
            },
            CpInfo::Module { name_index } => Self::Module {
                name: ConstantPoolIndex::new(*name_index),
            },
            CpInfo::Package { name_index } => Self::Package {
                name: ConstantPoolIndex::new(*name_index),
            },
        }
    }
}
