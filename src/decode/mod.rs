use rust_asm::class_reader::ClassReader;

use crate::error::FerroBabeError;
use crate::input::{ClassHeader, MAX_CLASS_MAJOR_VERSION};
use crate::model::Class;

const RUST_ASM_MAX_CLASS_MAJOR_VERSION: u16 = 69;

pub(crate) fn decode_class(bytes: &[u8], header: ClassHeader) -> Result<Class, FerroBabeError> {
    let adjusted_bytes = adjust_version_for_rust_asm(bytes, header);
    let mut node = ClassReader::new(&adjusted_bytes)
        .to_class_node()
        .map_err(|source| FerroBabeError::Decode { source })?;

    node.major_version = header.major_version;
    node.minor_version = header.minor_version;

    Ok(Class::from_node(node))
}

fn adjust_version_for_rust_asm(bytes: &[u8], header: ClassHeader) -> Vec<u8> {
    if header.major_version < MAX_CLASS_MAJOR_VERSION {
        return bytes.to_vec();
    }

    let mut adjusted = bytes.to_vec();
    adjusted[6..8].copy_from_slice(&RUST_ASM_MAX_CLASS_MAJOR_VERSION.to_be_bytes());
    adjusted
}
