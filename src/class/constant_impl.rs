use std::fmt::{Display, Error, Formatter};

type FmtResult<T = ()> = Result<T, Error>;

use crate::descriptor::{AnyDescriptor, FieldDescriptor, MethodDescriptor};

use super::{
    ConstIndex,
    Constant,
};

#[must_use]
pub struct DisplayConstant<'a>(pub(super) ConstIndex, pub(super) &'a [Constant]);
#[must_use]
pub struct DisplayFieldDescriptor<'a>(pub(super) ConstIndex, pub(super) &'a [Constant]);
#[must_use]
pub struct DisplayMethodDescriptor<'a>(pub(super) ConstIndex, pub(super) ConstIndex, pub(super) &'a [Constant]);
#[must_use]
pub struct DisplayDescriptor<'a>(pub(super) ConstIndex, pub(super) ConstIndex, pub(super) &'a [Constant]);

impl Display for DisplayConstant<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write_cpn(f, self.0, self.1)
    }
}
impl Display for DisplayFieldDescriptor<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write_field_descriptor(f, self.0, self.1)
    }
}
impl Display for DisplayMethodDescriptor<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write_imethod_descriptor(f, self.0, self.1, self.2)
    }
}
impl Display for DisplayDescriptor<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write_descriptor(f, self.0, self.1, self.2)
    }
}

fn get_constant(n: ConstIndex, constant_pool: &[Constant]) -> &Constant {
    &constant_pool[n as usize - 1]
}
fn get_constant_utf8(n: ConstIndex, constant_pool: &[Constant]) -> FmtResult<&str> {
    match get_constant(n, constant_pool) {
        Constant::Utf8(s) => Ok(s),
        _ => Err(Error),
    }
}

fn write_cpn(f: &mut Formatter, n: ConstIndex, constant_pool: &[Constant]) -> FmtResult {
    write_constant(f, get_constant(n, constant_pool), constant_pool)
}
fn write_field_descriptor(f: &mut Formatter, n: ConstIndex, constant_pool: &[Constant]) -> FmtResult {
    let fd = FieldDescriptor::from_bytes(get_constant_utf8(n, constant_pool)?.as_bytes()).unwrap();
    write!(f, "{}", fd.display_type())
}
fn write_imethod_descriptor(f: &mut Formatter, name: ConstIndex, descriptor: ConstIndex, constant_pool: &[Constant]) -> FmtResult {
    let name = get_constant_utf8(name, constant_pool)?;
    write_method_descriptor(f, name, descriptor, constant_pool)
}
fn write_method_descriptor(f: &mut Formatter, name: &str, descriptor: ConstIndex, constant_pool: &[Constant]) -> FmtResult {
    let fd = MethodDescriptor::from_bytes(get_constant_utf8(descriptor, constant_pool)?.as_bytes()).map_err(|_| Error)?;
    write!(f, "{}", fd.display_type(name))
}
fn write_descriptor(f: &mut Formatter, name: ConstIndex, descriptor: ConstIndex, constant_pool: &[Constant]) -> FmtResult {
    let name = get_constant_utf8(name, constant_pool)?;
    let fd = AnyDescriptor::from_bytes(get_constant_utf8(descriptor, constant_pool)?.as_bytes()).map_err(|_| Error)?;
    write!(f, "{}", fd.display_type(name))
}
fn write_constant(f: &mut Formatter, constant: &Constant, constant_pool: &[Constant]) -> FmtResult {
    match *constant {
        Constant::Gap => Ok(()),
        Constant::Class { name_index } => {
            write!(f, "Class ")?;
            write_cpn(f, name_index, constant_pool)
        }
        Constant::Fieldref {
            class_index,
            name_and_type_index,
        } => {
            write!(f, "Fieldref ")?;
            write_cpn(f, class_index, constant_pool)?;
            write!(f, ".")?;
            write_cpn(f, name_and_type_index, constant_pool)
        }
        Constant::Methodref {
            class_index,
            name_and_type_index,
        } => {
            write!(f, "Methodref ")?;
            write_cpn(f, class_index, constant_pool)?;
            write!(f, ".")?;
            write_cpn(f, name_and_type_index, constant_pool)
        }
        Constant::InterfaceMethodref {
            class_index,
            name_and_type_index,
        } => {
            write!(f, "InterfaceMethodref ")?;
            write_cpn(f, class_index, constant_pool)?;
            write!(f, ".")?;
            write_cpn(f, name_and_type_index, constant_pool)
        },
        Constant::String { string_index } => {
            write!(f, "String ")?;
            write_cpn(f, string_index, constant_pool)
        }
        Constant::Integer { bytes } => write!(f, "Integer = {bytes}"),
        Constant::Float { bytes } => write!(f, "Float = {bytes:04x}"),
        Constant::Long {
            high_bytes,
            low_bytes,
        } => write!(f, "Long = {high_bytes:04x} {low_bytes:04x}"),
        Constant::Double {
            high_bytes,
            low_bytes,
        } => write!(f, "Double = {high_bytes:04x} {low_bytes:04x}"),
        Constant::NameAndType {
            name_index,
            descriptor_index,
        } => {
            write_descriptor(f, name_index, descriptor_index, constant_pool)
        },
        Constant::Utf8(ref s) => write!(f, "{s:?}"),
        Constant::MethodHandle {
            reference_kind,
            reference_index,
        } => {
            write!(f, "MethodHandle reference_kind = {reference_kind} reference = ")?;
            write_cpn(f, reference_index, constant_pool)
        }
        Constant::MethodType { descriptor_index } => {
            write!(f, "MethodType ")?;
            write_method_descriptor(f, "", descriptor_index, constant_pool)
        }
        Constant::Dynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        } => {
            write!(f, "Dynamic bootstrap_method_attr = {bootstrap_method_attr_index} ")?;
            write_cpn(f, name_and_type_index, constant_pool)
        },
        Constant::InvokeDynamic {
            bootstrap_method_attr_index,
            name_and_type_index,
        } => {
            write!(f, "InvokeDynamic bootstrap_method_attr = {bootstrap_method_attr_index} ")?;
            write_cpn(f, name_and_type_index, constant_pool)
        },
        Constant::Module { name_index } => {
            write!(f, "Module ")?;
            write_cpn(f, name_index, constant_pool)
        },
        Constant::Package { name_index } => {
            write!(f, "Package ")?;
            write_cpn(f, name_index, constant_pool)
        },
    }
}
