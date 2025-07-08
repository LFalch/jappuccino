use std::{fmt::{self, Debug, Display}, io::{self, Read}};

use bitflags::bitflags;
use collect_result::CollectResult;

use crate::{code::Code, descriptor::{FieldDescriptor, MethodDescriptor}, modified_utf8::read_modified_utf8, ReadIntExt};

pub type ConstIndex = u16;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassFile {
    pub version: (u16, u16),

    pub constant_pool: Box<[Constant]>,
    pub access_flags: ClassAccess,
    pub this_class: u16,
    pub super_class: u16,

    pub interfaces: Box<[ConstIndex]>,
    pub fields: Box<[Field]>,
    pub methods: Box<[Method]>,
    pub attributes: Box<[AttributeInfo]>,
}

impl ClassFile {
    pub fn constant(&self, n: ConstIndex) -> &Constant {
        &self.constant_pool[(n - 1) as usize]
    }
    pub fn constant_utf8(&self, n: ConstIndex) -> Option<&str> {
        match *self.constant(n) {
            Constant::Utf8(ref s) => Some(s),
            _ => None,
        }
    }
    pub fn constant_class(&self, n: ConstIndex) -> Option<&str> {
        match *self.constant(n) {
            Constant::Class{name_index} => self.constant_utf8(name_index),
            _ => None,
        }
    }
    pub fn constant_fdescriptor(&self, n: ConstIndex) -> Option<FieldDescriptor> {
        FieldDescriptor::from_bytes(self.constant_utf8(n)?.as_bytes()).ok()
    }
    pub fn constant_mdescriptor(&self, n: ConstIndex) -> Option<MethodDescriptor> {
        MethodDescriptor::from_bytes(self.constant_utf8(n)?.as_bytes()).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constant {
    Class {
        name_index: ConstIndex,
    },
    Fieldref {
        class_index: ConstIndex,
        name_and_type_index: ConstIndex,
    },
    Methodref {
        class_index: ConstIndex,
        name_and_type_index: ConstIndex,
    },
    InterfaceMethodref {
        class_index: ConstIndex,
        name_and_type_index: ConstIndex,
    },
    String {
        string_index: ConstIndex,
    },
    Integer {
        bytes: u32,
    },
    Float {
        bytes: u32,
    },
    Long {
        high_bytes: u32,
        low_bytes: u32,
    },
    Double {
        high_bytes: u32,
        low_bytes: u32,
    },
    NameAndType {
        name_index: ConstIndex,
        descriptor_index: ConstIndex,
    },
    Utf8(Box<str>),
    MethodHandle {
        reference_kind: u8,
        reference_index: ConstIndex,
    },
    MethodType {
        descriptor_index: ConstIndex,
    },
    Dynamic {
        bootstrap_method_attr_index: ConstIndex,
        name_and_type_index: ConstIndex,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: ConstIndex,
        name_and_type_index: ConstIndex,
    },
    Module {
        name_index: ConstIndex,
    },
    Package {
        name_index: ConstIndex,
    },
    /// usually low bytes of a long or double
    Gap,
}
impl Constant {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let tag = reader.read_u8()?;

        Ok(match tag {
            7 => Self::Class {
                name_index: reader.read_u16()?,
            },
            9 => Self::Fieldref {
                class_index: reader.read_u16()?,
                name_and_type_index: reader.read_u16()?,
            },
            10 => Self::Methodref {
                class_index: reader.read_u16()?,
                name_and_type_index: reader.read_u16()?,
            },
            11 => Self::InterfaceMethodref {
                class_index: reader.read_u16()?,
                name_and_type_index: reader.read_u16()?,
            },
            8 => Self::String {
                string_index: reader.read_u16()?,
            },
            3 => Self::Integer {
                bytes: reader.read_u32()?,
            },
            4 => Self::Float {
                bytes: reader.read_u32()?,
            },
            5 => Self::Long {
                high_bytes: reader.read_u32()?,
                low_bytes: reader.read_u32()?,
            },
            6 => Self::Double {
                high_bytes: reader.read_u32()?,
                low_bytes: reader.read_u32()?,
            },
            12 => Self::NameAndType {
                name_index: reader.read_u16()?,
                descriptor_index: reader.read_u16()?,
            },
            1 => Self::Utf8 ({
                let length = reader.read_u16()?;
                read_modified_utf8(reader, length as usize)?.into_boxed_str()
            }),
            15 => Self::MethodHandle {
                reference_kind: reader.read_u8()?,
                reference_index: reader.read_u16()?,
            },
            16 => Self::MethodType {
                descriptor_index: reader.read_u16()?,
            },
            17 => Self::Dynamic {
                bootstrap_method_attr_index: reader.read_u16()?,
                name_and_type_index: reader.read_u16()?,
            },
            18 => Self::InvokeDynamic {
                bootstrap_method_attr_index: reader.read_u16()?,
                name_and_type_index: reader.read_u16()?,
            },
            19 => Self::Module {
                name_index: reader.read_u16()?,
            },
            20 => Self::Package {
                name_index: reader.read_u16()?,
            },
            n => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown constant tag {n}"))),
        })
    }
    const fn is_wide(&self) -> bool {
        match self {
            Self::Double { .. } | Self::Long { .. } => true,
            _ => false,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub access_flags: FieldAccess,
    pub name_index: ConstIndex,
    pub descriptor_index: ConstIndex,
    pub attributes: Box<[AttributeInfo]>,
}
impl Field {
    fn read<R: Read>(reader: &mut R, constant_pool: &[Constant]) -> io::Result<Self> {
        Ok(Self {
            access_flags: FieldAccess::from_bits_retain(reader.read_u16()?),
            name_index: reader.read_u16()?,
            descriptor_index: reader.read_u16()?,
            attributes: {
                let attributes_count = reader.read_u16()?;
                let attributes: Vec<_> = (0..attributes_count).map(|_| {
                    let attrib = RawAttribute::read(reader)?;
                    AttributeInfo::from_raw_attribute(attrib, constant_pool)
                }).collect_result()?;
                attributes.into_boxed_slice()
            }
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Method {
    pub access_flags: MethodAccess,
    pub name_index: ConstIndex,
    pub descriptor_index: ConstIndex,
    pub attributes: Box<[AttributeInfo]>,
}
impl Method {
    fn read<R: Read>(reader: &mut R, constant_pool: &[Constant]) -> io::Result<Self> {
        Ok(Self {
            access_flags: MethodAccess::from_bits_retain(reader.read_u16()?),
            name_index: reader.read_u16()?,
            descriptor_index: reader.read_u16()?,
            attributes: {
                let attributes_count = reader.read_u16()?;
                let attributes: Vec<_> = (0..attributes_count).map(|_| {
                    let attrib = RawAttribute::read(reader)?;
                    AttributeInfo::from_raw_attribute(attrib, constant_pool)
                }).collect_result()?;
                attributes.into_boxed_slice()
            }
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct RawAttribute {
    pub attribute_name_index: ConstIndex,
    pub info: Box<[u8]>,
}
impl RawAttribute {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            attribute_name_index: reader.read_u16()?,
            info: {
                let attribute_length = reader.read_u32()?;
                let info = reader.read_bytes(attribute_length as usize)?;
                info
            }
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeInfo {
    ConstantValue {
        constantvalue_index: ConstIndex,
    },
    Code {
        max_stack: u16,
        max_locals: u16,
        /// Raw bytecode
        code: Code,
        exception_table: Box<[ExceptionEntry]>,
        attributes: Box<[AttributeInfo]>,
    },
    StackMapTable(Box<[StackMapFrame]>),
    Exceptions(RawBytes),
    InnerClasses(Box<[InnerClass]>),
    EnclosingMethod(RawBytes),
    Synthetic(RawBytes),
    Signature(RawBytes),
    SourceFile {
        sourcefile_index: u16,
    },
    SourceDebugExtension(RawBytes),
    LineNumberTable(Box<[LineNumberEntry]>),
    LocalVariableTable(Box<[LocalVariableEntry]>),
    LocalVariableTypeTable(RawBytes),
    Deprecated(RawBytes),
    RuntimeVisibleAnnotations(RawBytes),
    RuntimeInvisibleAnnotations(RawBytes),
    RuntimeVisibleParameterAnnotations(RawBytes),
    RuntimeInvisibleParameterAnnotations(RawBytes),
    RuntimeVisibleTypeAnnotations(RawBytes),
    RuntimeInvisibleTypeAnnotations(RawBytes),
    AnnotationDefault(RawBytes),
    BootstrapMethods(Box<[BootstrapMethod]>),
    NestHost(RawBytes),
    NestMembers(RawBytes),
    PermittedSubclasses(RawBytes),
    MethodParameters(RawBytes),
    Module(RawBytes),
    ModulePackages(RawBytes),
    ModuleMainClass(RawBytes),
}
impl AttributeInfo {
    fn from_raw_attribute(attrib: RawAttribute, constant_pool: &[Constant]) -> io::Result<Self> {
        let RawAttribute { attribute_name_index, info } = attrib;
        let Constant::Utf8(name) = &constant_pool[attribute_name_index as usize - 1] else {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "attribute name was not a utf8 constant"));
        };
        Self::from_info(name, info, constant_pool)
    }
    fn from_info(name: &str, info: Box<[u8]>, constant_pool: &[Constant]) -> io::Result<Self> {
        let mut reader = &*info;
        Ok(match name {
            "ConstantValue" => Self::ConstantValue {
                constantvalue_index: reader.read_u16()?,
            },
            "Code" => Self::Code {
                max_stack: reader.read_u16()?,
                max_locals: reader.read_u16()?,
                code: {
                    let code_length = reader.read_u32()?;
                    Code(reader.read_bytes(code_length as usize)?)
                },
                exception_table: {
                    let exception_table_length = reader.read_u16()?;
                    let exception_table: Vec<_> = (0..exception_table_length).map(|_| -> io::Result<_> {
                        Ok(ExceptionEntry {
                            start_pc: reader.read_u16()?,
                            end_pc: reader.read_u16()?,
                            handler_pc: reader.read_u16()?,
                            catch_type: reader.read_u16()?,
                        })
                    }).collect_result()?;
                    exception_table.into_boxed_slice()
                },
                attributes: {
                    let attributes_count = reader.read_u16()?;
                    let attributes: Vec<_> = (0..attributes_count).map(|_| {
                        let attrib = RawAttribute::read(&mut reader)?;
                        AttributeInfo::from_raw_attribute(attrib, constant_pool)
                    }).collect_result()?;
                    attributes.into_boxed_slice()
                },
            },
            "StackMapTable" => Self::StackMapTable({
                let number_of_entries = reader.read_u16()?;
                let entries: Vec<_> = (0..number_of_entries)
                    .map(|_| StackMapFrame::read(&mut reader))
                    .collect_result()?;
                entries.into_boxed_slice()
            }),
            "Exceptions" => Self::Exceptions(RawBytes(info)),
            "InnerClasses" => Self::InnerClasses({
                let number_of_classes = reader.read_u16()?;
                let classes: Vec<_> = (0..number_of_classes).map(|_| -> io::Result<_> {
                    Ok(InnerClass {
                        inner_class_info_index: reader.read_u16()?,
                        outer_class_info_index: reader.read_u16()?,
                        inner_name_index: reader.read_u16()?,
                        inner_class_access_flags: InnerClassAccess::from_bits_retain(reader.read_u16()?),
                    })
                }).collect_result()?;
                classes.into_boxed_slice()
            }),
            "EnclosingMethod" => Self::EnclosingMethod(RawBytes(info)),
            "Synthetic" => Self::Synthetic(RawBytes(info)),
            "Signature" => Self::Signature(RawBytes(info)),
            "SourceFile" => Self::SourceFile {
                sourcefile_index: reader.read_u16()?,
            },
            "SourceDebugExtension" => Self::SourceDebugExtension(RawBytes(info)),
            "LineNumberTable" => Self::LineNumberTable({
                let line_number_table_length = reader.read_u16()?;
                let line_number_table: Vec<_> = (0..line_number_table_length).map(|_| -> io::Result<_> {
                    Ok(LineNumberEntry {
                        start_pc: reader.read_u16()?,
                        line_number: reader.read_u16()?,
                    })
                }).collect_result()?;
                line_number_table.into_boxed_slice()
            }),
            "LocalVariableTable" => Self::LocalVariableTable({
                let local_variable_table_length = reader.read_u16()?;
                let local_variable_table: Vec<_> = (0..local_variable_table_length).map(|_| -> io::Result<_> {
                    Ok(LocalVariableEntry {
                        start_pc: reader.read_u16()?,
                        length: reader.read_u16()?,
                        name_index: reader.read_u16()?,
                        descriptor_index: reader.read_u16()?,
                        index: reader.read_u16()?,
                    })
                }).collect_result()?;
                local_variable_table.into_boxed_slice()
            }),
            "LocalVariableTypeTable" => Self::LocalVariableTypeTable(RawBytes(info)),
            "Deprecated" => Self::Deprecated(RawBytes(info)),
            "RuntimeVisibleAnnotations" => Self::RuntimeVisibleAnnotations(RawBytes(info)),
            "RuntimeInvisibleAnnotations" => Self::RuntimeInvisibleAnnotations(RawBytes(info)),
            "RuntimeVisibleParameterAnnotations" => Self::RuntimeVisibleParameterAnnotations(RawBytes(info)),
            "RuntimeInvisibleParameterAnnotations" => Self::RuntimeInvisibleParameterAnnotations(RawBytes(info)),
            "RuntimeVisibleTypeAnnotations" => Self::RuntimeVisibleTypeAnnotations(RawBytes(info)),
            "RuntimeInvisibleTypeAnnotations" => Self::RuntimeInvisibleTypeAnnotations(RawBytes(info)),
            "AnnotationDefault" => Self::AnnotationDefault(RawBytes(info)),
            "BootstrapMethods" => Self::BootstrapMethods({
                let num_bootstrap_methods = reader.read_u16()?;
                let bootstrap_methods: Vec<_> = (0..num_bootstrap_methods).map(|_| -> io::Result<_> {
                    Ok(BootstrapMethod {
                        bootstrap_method_ref: reader.read_u16()?,
                        bootstrap_arguments: {
                            let num_bootstrap_arguments = reader.read_u16()?;
                            (0..num_bootstrap_arguments)
                                .map(|_| reader.read_u16())
                                .collect_result::<Vec<_>>()?
                                .into_boxed_slice()
                        }
                    })
                }).collect_result()?;
                bootstrap_methods.into_boxed_slice()
            }),
            "NestHost" => Self::NestHost(RawBytes(info)),
            "NestMembers" => Self::NestMembers(RawBytes(info)),
            "PermittedSubclasses" => Self::PermittedSubclasses(RawBytes(info)),
            "MethodParameters" => Self::MethodParameters(RawBytes(info)),
            "Module" => Self::Module(RawBytes(info)),
            "ModulePackages" => Self::ModulePackages(RawBytes(info)),
            "ModuleMainClass" => Self::ModuleMainClass(RawBytes(info)),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("unknown attribute {name}"))),
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LineNumberEntry {
    pub start_pc: u16,
    pub line_number: u16,
}
#[derive(Debug, Clone, PartialEq)]
pub struct LocalVariableEntry {
    pub start_pc: u16,
    pub length: u16,
    pub name_index: u16,
    pub descriptor_index: ConstIndex,
    pub index: u16,
}
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitializedThis,
    Object(ConstIndex),
    Uninitialized {
        offset: u16
    },
}
impl VerificationTypeInfo {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let tag = reader.read_u8()?;
        Ok(match tag {
            0 => Self::Top,
            1 => Self::Integer,
            2 => Self::Float,
            3 => Self::Double,
            4 => Self::Long,
            5 => Self::Null,
            6 => Self::UninitializedThis,
            7 => Self::Object(reader.read_u16()?),
            8 => Self::Uninitialized {
                offset: reader.read_u16()?,
            },
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unknown verification type info tag")),
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum StackMapFrame {
    SameFrame {
        offset_delta: u8,
    },
    SameLocals1StackItemFrame {
        offset_delta: u8,
        stack: VerificationTypeInfo,
    },
    SameLocals1StackItemFrameExtended {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    },
    ChopFrame {
        k: u8,
        offset_delta: u16,
    },
    SameFrameExtended {
        offset_delta: u16,
    },
    AppendFrame {
        offset_delta: u16,
        locals: Box<[VerificationTypeInfo]>,
    },
    FullFrame {
        offset_delta: u16,
        locals: Box<[VerificationTypeInfo]>,
        stack: Box<[VerificationTypeInfo]>,
    }
}
impl StackMapFrame {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let frame_type = reader.read_u8()?;
        Ok(match frame_type {
            0..=63 => Self::SameFrame { offset_delta: frame_type },
            64..=127 => Self::SameLocals1StackItemFrame {
                offset_delta: frame_type - 64,
                stack: VerificationTypeInfo::read(reader)?,
            },
            128..=246 => return Err(io::Error::new(io::ErrorKind::InvalidData, "reserved frame type value")),
            247 => Self::SameLocals1StackItemFrameExtended {
                offset_delta: reader.read_u16()?,
                stack: VerificationTypeInfo::read(reader)?,
            },
            248..=250 => Self::ChopFrame {
                k: 251 - frame_type,
                offset_delta: reader.read_u16()?,
            },
            251 => Self::SameFrameExtended {
                offset_delta: reader.read_u16()?,
            },
            252..=254 => Self::AppendFrame {
                offset_delta: reader.read_u16()?,
                locals: (0..frame_type - 251)
                    .map(|_| VerificationTypeInfo::read(reader))
                    .collect_result::<Vec<_>>()?
                    .into_boxed_slice(),
            },
            255 => {
                let offset_delta = reader.read_u16()?;
                let number_of_locals = reader.read_u16()?;
                let locals =  (0..number_of_locals)
                    .map(|_| VerificationTypeInfo::read(reader))
                    .collect_result::<Vec<_>>()?
                    .into_boxed_slice();
                let number_of_stack_items = reader.read_u16()?;
                let stack =  (0..number_of_stack_items)
                    .map(|_| VerificationTypeInfo::read(reader))
                    .collect_result::<Vec<_>>()?
                    .into_boxed_slice();
                Self::FullFrame {
                    offset_delta,
                    locals,
                    stack,
                }
            }
        })
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InnerClass {
    inner_class_info_index: ConstIndex,
    outer_class_info_index: ConstIndex,
    inner_name_index: ConstIndex,
    inner_class_access_flags: InnerClassAccess,
}
#[derive(Debug, Clone, PartialEq)]
pub struct BootstrapMethod {
    bootstrap_method_ref: ConstIndex,
    bootstrap_arguments: Box<[ConstIndex]>,
}
#[derive(Clone, PartialEq)]
pub struct RawBytes(pub Box<[u8]>);
impl Debug for RawBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RawBytes[")?;
        for byte in &self.0 {
            write!(f, "{byte:02X}")?;
        }
        write!(f, "]")
    }
}
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FieldAccess: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM = 0x4000;
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MethodAccess: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABSTRACT = 0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ClassAccess: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
        const MODULE = 0x8000;
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InnerClassAccess: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
    }
}
impl Display for FieldAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut space = "";
        for (name, _) in self.iter_names() {
            // this is awful
            write!(f, "{space}{}", name.to_lowercase())?;
            space = " ";
        }
        if space == "" {
            write!(f, "bare")?;
        }
        Ok(())
    }
}
impl Display for MethodAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut space = "";
        for (name, _) in self.iter_names() {
            // this is awful
            write!(f, "{space}{}", name.to_lowercase())?;
            space = " ";
        }
        if space == "" {
            write!(f, "bare")?;
        }
        Ok(())
    }
}
impl Display for ClassAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut space = "";
        for (name, _) in self.iter_names() {
            // this is awful
            write!(f, "{space}{}", name.to_lowercase())?;
            space = " ";
        }
        if space == "" {
            write!(f, "bare")?;
        }
        Ok(())
    }
}

impl ClassFile {
    pub fn from_reader<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut magic = [0; 4];
        reader.read_exact(&mut magic)?;
        if magic != [0xCA, 0xFE, 0xBA, 0xBE] {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "magic bytes did not match"));
        }
        let minor = reader.read_u16()?;
        let major = reader.read_u16()?;
        let version = (major, minor);
        // TODO: check if the version is supported

        let constant_pool_count = reader.read_u16()?;
        let mut was_wide = false;
        // TODO: fix this bad hack
        let constant_pool: Vec<_> = (1..constant_pool_count).map(|_| -> io::Result<Constant> {
            if was_wide {
                was_wide = false;
                Ok(Constant::Gap)
            } else {
                let c = Constant::read(&mut reader)?;
                was_wide = c.is_wide();
                Ok(c)
            }
        }).collect_result()?;
        let access_flags = reader.read_u16()?;
        let this_class = reader.read_u16()?;
        let super_class = reader.read_u16()?;
        let interfaces_count = reader.read_u16()?;
        let interfaces: Vec<_> = (0..interfaces_count).map(|_| reader.read_u16()).collect_result()?;
        let fields_count = reader.read_u16()?;
        let fields: Vec<_> = (0..fields_count).map(|_| Field::read(&mut reader, &constant_pool)).collect_result()?;
        let methods_count = reader.read_u16()?;
        let methods: Vec<_> = (0..methods_count).map(|_| Method::read(&mut reader, &constant_pool)).collect_result()?;
        let attributes = {
            let attributes_count = reader.read_u16()?;
            let attributes: Vec<_> = (0..attributes_count).map(|_| {
                let attrib = RawAttribute::read(&mut reader)?;
                AttributeInfo::from_raw_attribute(attrib, &constant_pool)
            }).collect_result()?;
            attributes.into_boxed_slice()
        };

        Ok(ClassFile {
            version,
            constant_pool: constant_pool.into_boxed_slice(),
            access_flags: ClassAccess::from_bits_retain(access_flags),
            this_class,
            super_class,
            interfaces: interfaces.into_boxed_slice(),
            fields: fields.into_boxed_slice(),
            methods: methods.into_boxed_slice(),
            attributes,
        })
    }
}