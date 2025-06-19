use std::{fmt::{self, Debug}, io::{self, Read}};

use bitflags::bitflags;
use collect_result::CollectResult;

pub type ConstIndex = u16;

trait ReadIntExt {
    fn read_u8(&mut self) -> io::Result<u8>;
    fn read_u16(&mut self) -> io::Result<u16>;
    fn read_u32(&mut self) -> io::Result<u32>;
    fn read_bytes(&mut self, n: usize) -> io::Result<Box<[u8]>>;
}
impl<R: Read> ReadIntExt for R {
    fn read_u8(&mut self) -> io::Result<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(u8::from_be_bytes(buf))
    }
    fn read_u16(&mut self) -> io::Result<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }
    fn read_u32(&mut self) -> io::Result<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }
    fn read_bytes(&mut self, n: usize) -> io::Result<Box<[u8]>> {
        let mut buf = vec![0; n];
        self.read_exact(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassFile {
    pub version: (u16, u16),

    pub constant_pool: Box<[Constant]>,
    pub access_flags: AccessFlags,
    pub this_class: u16,
    pub super_class: u16,

    pub interfaces: Box<[ConstIndex]>,
    pub fields: Box<[Field]>,
    pub methods: Box<[Method]>,
    pub attributes: Box<[AttributeInfo]>,
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
                let mut string = String::with_capacity(length as usize);
                for _ in 0..length {
                    let b = reader.read_u8()?;
                    match b {
                        1..=0x7f => string.push(b as char),
                        _ => todo!(),
                    }
                }
                string.into_boxed_str()
            }),
            15 => Self::MethodHandle {
                reference_kind: reader.read_u8()?,
                reference_index: reader.read_u16()?,
            },
            16 => Self::MethodType {
                descriptor_index: reader.read_u16()?,
            },
            18 => Self::InvokeDynamic {
                name_and_type_index: reader.read_u16()?,
                bootstrap_method_attr_index: reader.read_u16()?,
            },
            19 => Self::Module {
                name_index: reader.read_u16()?,
            },
            20 => Self::Package {
                name_index: reader.read_u16()?,
            },
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "unknown class tag")),
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub access_flags: AccessFlags,
    pub name_index: ConstIndex,
    pub descriptor_index: ConstIndex,
    pub attributes: Box<[AttributeInfo]>,
}
impl Field {
    fn read<R: Read>(reader: &mut R, constant_pool: &[Constant]) -> io::Result<Self> {
        Ok(Self {
            access_flags: AccessFlags::from_bits_retain(reader.read_u16()?),
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
    pub access_flags: AccessFlags,
    pub name_index: ConstIndex,
    pub descriptor_index: ConstIndex,
    pub attributes: Box<[AttributeInfo]>,
}
impl Method {
    fn read<R: Read>(reader: &mut R, constant_pool: &[Constant]) -> io::Result<Self> {
        Ok(Self {
            access_flags: AccessFlags::from_bits_retain(reader.read_u16()?),
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
        code: RawBytes,
        exception_table: Box<[ExceptionEntry]>,
        attributes: Box<[AttributeInfo]>,
    },
    // StackMapTable,
    // Exceptions,
    // InnerClasses,
    // EnclosingMethod,
    // Synthetic,
    // Signature,
    SourceFile {
        sourcefile_index: u16,
    },
    // SourceDebugExtension,
    LineNumberTable(Box<[LineNumberEntry]>),
    // LocalVariableTable,
    // LocalVariableTypeTable,
    // Deprecated,
    // RuntimeVisibleAnnotations,
    // RuntimeInvisibleAnnotations,
    // RuntimeVisibleParameterAnnotations,
    // RuntimeInvisibleParameterAnnotations,
    // RuntimeVisibleTypeAnnotations,
    // RuntimeInvisibleTypeAnnotations,
    // AnnotationDefault,
    // BootstrapMethods,
    // MethodParameters,
    // Module,
    // ModulePackages,
    // ModuleMainClass,

    Unknown(Box<str>, RawBytes),
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
                    RawBytes::read(&mut reader, code_length as usize)?
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
            // "StackMapTable" => Self::StackMapTable,
            // "Exceptions" => Self::Exceptions,
            // "InnerClasses" => Self::InnerClasses,
            // "EnclosingMethod" => Self::EnclosingMethod,
            // "Synthetic" => Self::Synthetic,
            // "Signature" => Self::Signature,
            "SourceFile" => Self::SourceFile {
                sourcefile_index: reader.read_u16()?,
            },
            // "SourceDebugExtension" => Self::SourceDebugExtension,
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
            // "LocalVariableTable" => Self::LocalVariableTable,
            // "LocalVariableTypeTable" => Self::LocalVariableTypeTable,
            // "Deprecated" => Self::Deprecated,
            // "RuntimeVisibleAnnotations" => Self::RuntimeVisibleAnnotations,
            // "RuntimeInvisibleAnnotations" => Self::RuntimeInvisibleAnnotations,
            // "RuntimeVisibleParameterAnnotations" => Self::RuntimeVisibleParameterAnnotations,
            // "RuntimeInvisibleParameterAnnotations" => Self::RuntimeInvisibleParameterAnnotations,
            // "RuntimeVisibleTypeAnnotations" => Self::RuntimeVisibleTypeAnnotations,
            // "RuntimeInvisibleTypeAnnotations" => Self::RuntimeInvisibleTypeAnnotations,
            // "AnnotationDefault" => Self::AnnotationDefault,
            // "BootstrapMethods" => Self::BootstrapMethods,
            // "MethodParameters" => Self::MethodParameters,
            // "Module" => Self::Module,
            // "ModulePackages" => Self::ModulePackages,
            // "ModuleMainClass" => Self::ModuleMainClass,
            _ => Self::Unknown(name.into(), RawBytes(info)),
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
#[derive(Clone, PartialEq)]
pub struct RawBytes(pub Box<[u8]>);
impl RawBytes {
    fn read<R: Read>(reader: &mut R, n: usize) -> io::Result<Self> {
        Ok(RawBytes(reader.read_bytes(n)?))
    }
}
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
    pub struct AccessFlags: u16 {
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
        let constant_pool: Vec<_> = (1..constant_pool_count).map(|_| Constant::read(&mut reader)).collect_result()?;
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
            access_flags: AccessFlags::from_bits_retain(access_flags),
            this_class,
            super_class,
            interfaces: interfaces.into_boxed_slice(),
            fields: fields.into_boxed_slice(),
            methods: methods.into_boxed_slice(),
            attributes,
        })
    }
}