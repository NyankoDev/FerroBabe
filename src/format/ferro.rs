use std::fmt;

use crate::model::{
    Class, ConstantPoolIndex, ConstantRef, Instruction, InstructionOperand, LdcValueRef,
    MemberReference,
};

use super::Formatter;
use super::opcode::mnemonic;

#[derive(Debug, Default, Clone, Copy)]
pub struct FerroFormatter;

impl Formatter for FerroFormatter {
    fn write_class(&self, class: &Class, output: &mut dyn fmt::Write) -> fmt::Result {
        write!(output, "class {}", class.name())?;
        if let Some(super_name) = class.super_name() {
            write!(output, " : {super_name}")?;
        }
        write!(output, " ")?;
        write_flags(output, class.access_flags(), FlagContext::Class)?;
        writeln!(
            output,
            " v{}.{}",
            class.version().major(),
            class.version().minor()
        )?;

        let interfaces: Vec<_> = class.interfaces().collect();
        if !interfaces.is_empty() {
            writeln!(output, "implements {}", interfaces.join(", "))?;
        }

        for field in class.fields() {
            write!(output, "\nfield {} {} ", field.name(), field.descriptor())?;
            write_flags(output, field.access_flags(), FlagContext::Field)?;
            writeln!(output)?;
        }

        for method in class.methods() {
            write!(output, "\nmethod {}{} ", method.name(), method.descriptor())?;
            write_flags(output, method.access_flags(), FlagContext::Method)?;

            if method.has_code() {
                writeln!(
                    output,
                    " stack={} locals={}",
                    method.max_stack(),
                    method.max_locals()
                )?;

                if let Some(instructions) = method.instructions() {
                    for instruction in instructions {
                        write_instruction(output, class, instruction)?;
                    }
                }

                for handler in method.exception_handlers() {
                    write!(
                        output,
                        "  catch {:04x}..{:04x} -> {:04x}",
                        handler.start().get(),
                        handler.end().get(),
                        handler.handler().get()
                    )?;
                    if let Some(catch_type) = handler.catch_type() {
                        write!(output, " {catch_type}")?;
                    }
                    writeln!(output)?;
                }
            } else {
                writeln!(output)?;
            }
        }

        Ok(())
    }
}

fn write_instruction(
    output: &mut dyn fmt::Write,
    class: &Class,
    instruction: Instruction<'_>,
) -> fmt::Result {
    write!(
        output,
        "  {:04x}  {:<16}",
        instruction.offset().get(),
        mnemonic(instruction.opcode())
    )?;
    write_operand(output, class, instruction.operand())?;
    writeln!(output)
}

fn write_operand(
    output: &mut dyn fmt::Write,
    class: &Class,
    operand: InstructionOperand<'_>,
) -> fmt::Result {
    match operand {
        InstructionOperand::None => Ok(()),
        InstructionOperand::Immediate(value) => write!(output, "{value}"),
        InstructionOperand::Local(index) => write!(output, "{index}"),
        InstructionOperand::ConstantPool(index) => write_constant(output, class, index),
        InstructionOperand::Member(reference) => write_member_reference(output, class, reference),
        InstructionOperand::InvokeInterface { method, count } => {
            write_constant(output, class, method)?;
            write!(output, " count={count}")
        }
        InstructionOperand::InvokeDynamic { call_site } => write_constant(output, class, call_site),
        InstructionOperand::Branch { target, .. } => write!(output, "{:04x}", target),
        InstructionOperand::Ldc(value) => write_ldc_value(output, class, value),
        InstructionOperand::Increment { local, amount } => write!(output, "{local} {amount}"),
        InstructionOperand::TableSwitch {
            default,
            low,
            high,
            targets,
            base_offset,
        } => {
            write!(output, "{low}..{high} default={:04x} [", default.target())?;
            for (index, target) in targets.iter().enumerate() {
                if index > 0 {
                    write!(output, ", ")?;
                }
                write!(output, "{:04x}", i32::from(base_offset.get()) + target)?;
            }
            write!(output, "]")
        }
        InstructionOperand::LookupSwitch {
            default,
            pairs,
            base_offset,
        } => {
            write!(output, "default={:04x} [", default.target())?;
            for (index, (key, target)) in pairs.iter().enumerate() {
                if index > 0 {
                    write!(output, ", ")?;
                }
                write!(
                    output,
                    "{key}:{:04x}",
                    i32::from(base_offset.get()) + target
                )?;
            }
            write!(output, "]")
        }
        InstructionOperand::MultiArray {
            class: index,
            dimensions,
        } => {
            write_constant(output, class, index)?;
            write!(output, " dims={dimensions}")
        }
    }
}

fn write_ldc_value(
    output: &mut dyn fmt::Write,
    class: &Class,
    value: LdcValueRef<'_>,
) -> fmt::Result {
    match value {
        LdcValueRef::ConstantPool(index) => write_constant(output, class, index),
        LdcValueRef::String(value) => write!(output, "{value:?}"),
        LdcValueRef::TypeDescriptor => write!(output, "<type>"),
        LdcValueRef::Integer(value) => write!(output, "{value}"),
        LdcValueRef::Float(value) => write!(output, "{value}"),
        LdcValueRef::Long(value) => write!(output, "{value}"),
        LdcValueRef::Double(value) => write!(output, "{value}"),
    }
}

fn write_member_reference(
    output: &mut dyn fmt::Write,
    class: &Class,
    reference: MemberReference<'_>,
) -> fmt::Result {
    match reference {
        MemberReference::ConstantPool(index) => write_constant(output, class, index),
        MemberReference::Symbolic {
            owner,
            name,
            descriptor,
        } => write!(output, "{owner}.{name}{descriptor}"),
    }
}

fn write_constant(
    output: &mut dyn fmt::Write,
    class: &Class,
    index: ConstantPoolIndex,
) -> fmt::Result {
    write!(output, "#{}", index.get())?;
    match class.constant(index) {
        Some(ConstantRef::Class { name }) => {
            if let Some(name) = utf8(class, name) {
                write!(output, " {name}")?;
            }
        }
        Some(ConstantRef::String { value }) => {
            if let Some(value) = utf8(class, value) {
                write!(output, " {value:?}")?;
            }
        }
        Some(ConstantRef::FieldReference {
            class: owner,
            name_and_type,
        })
        | Some(ConstantRef::MethodReference {
            class: owner,
            name_and_type,
        })
        | Some(ConstantRef::InterfaceMethodReference {
            class: owner,
            name_and_type,
        }) => write_member_constant(output, class, owner, name_and_type)?,
        Some(ConstantRef::Integer(value)) => write!(output, " {value}")?,
        Some(ConstantRef::Float(value)) => write!(output, " {value}")?,
        Some(ConstantRef::Long(value)) => write!(output, " {value}")?,
        Some(ConstantRef::Double(value)) => write!(output, " {value}")?,
        _ => {}
    }
    Ok(())
}

fn write_member_constant(
    output: &mut dyn fmt::Write,
    class: &Class,
    owner: ConstantPoolIndex,
    name_and_type_index: ConstantPoolIndex,
) -> fmt::Result {
    let Some(owner) = class_name(class, owner) else {
        return Ok(());
    };
    let Some((name, descriptor)) = name_and_type(class, name_and_type_index) else {
        return Ok(());
    };
    write!(output, " {owner}.{name}{descriptor}")
}

fn class_name(class: &Class, index: ConstantPoolIndex) -> Option<&str> {
    let ConstantRef::Class { name } = class.constant(index)? else {
        return None;
    };
    utf8(class, name)
}

fn name_and_type(class: &Class, index: ConstantPoolIndex) -> Option<(&str, &str)> {
    let ConstantRef::NameAndType { name, descriptor } = class.constant(index)? else {
        return None;
    };
    Some((utf8(class, name)?, utf8(class, descriptor)?))
}

fn utf8(class: &Class, index: ConstantPoolIndex) -> Option<&str> {
    let ConstantRef::Utf8(value) = class.constant(index)? else {
        return None;
    };
    Some(value)
}

#[derive(Clone, Copy)]
enum FlagContext {
    Class,
    Field,
    Method,
}

fn write_flags(output: &mut dyn fmt::Write, flags: u16, context: FlagContext) -> fmt::Result {
    write!(output, "[")?;
    let mut first = true;

    for (bit, name) in flag_names(context) {
        if flags & bit != 0 {
            if !first {
                write!(output, " ")?;
            }
            write!(output, "{name}")?;
            first = false;
        }
    }

    write!(output, "]")
}

fn flag_names(context: FlagContext) -> &'static [(u16, &'static str)] {
    const CLASS_FLAGS: &[(u16, &str)] = &[
        (0x0001, "public"),
        (0x0010, "final"),
        (0x0020, "super"),
        (0x0200, "interface"),
        (0x0400, "abstract"),
        (0x1000, "synthetic"),
        (0x2000, "annotation"),
        (0x4000, "enum"),
        (0x8000, "module"),
    ];
    const FIELD_FLAGS: &[(u16, &str)] = &[
        (0x0001, "public"),
        (0x0002, "private"),
        (0x0004, "protected"),
        (0x0008, "static"),
        (0x0010, "final"),
        (0x0040, "volatile"),
        (0x0080, "transient"),
        (0x1000, "synthetic"),
        (0x4000, "enum"),
    ];
    const METHOD_FLAGS: &[(u16, &str)] = &[
        (0x0001, "public"),
        (0x0002, "private"),
        (0x0004, "protected"),
        (0x0008, "static"),
        (0x0010, "final"),
        (0x0020, "synchronized"),
        (0x0040, "bridge"),
        (0x0080, "varargs"),
        (0x0100, "native"),
        (0x0400, "abstract"),
        (0x0800, "strict"),
        (0x1000, "synthetic"),
    ];

    match context {
        FlagContext::Class => CLASS_FLAGS,
        FlagContext::Field => FIELD_FLAGS,
        FlagContext::Method => METHOD_FLAGS,
    }
}
