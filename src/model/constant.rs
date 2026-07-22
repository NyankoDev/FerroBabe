use rust_asm::constant_pool::CpInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantPoolIndex(u16);

impl ConstantPoolIndex {
    pub(crate) const fn new(index: u16) -> Self {
        Self(index)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConstantRef<'a> {
    Unusable,
    Utf8(&'a str),
    Integer(i32),
    Float(f32),
    Long(i64),
    Double(f64),
    Class {
        name: ConstantPoolIndex,
    },
    String {
        value: ConstantPoolIndex,
    },
    FieldReference {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    MethodReference {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    InterfaceMethodReference {
        class: ConstantPoolIndex,
        name_and_type: ConstantPoolIndex,
    },
    NameAndType {
        name: ConstantPoolIndex,
        descriptor: ConstantPoolIndex,
    },
    MethodHandle {
        reference_kind: u8,
        reference: ConstantPoolIndex,
    },
    MethodType {
        descriptor: ConstantPoolIndex,
    },
    Dynamic {
        bootstrap_method: u16,
        name_and_type: ConstantPoolIndex,
    },
    InvokeDynamic {
        bootstrap_method: u16,
        name_and_type: ConstantPoolIndex,
    },
    Module {
        name: ConstantPoolIndex,
    },
    Package {
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
