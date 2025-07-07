use std::{error::Error, fmt::{self, Display}, io};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptorError {
    Empty,
    InfiniteClassName,
    UnknownType,
    InvalidUtf8,
    NoArguments,
    InfiniteArguments,
}
impl From<DescriptorError> for io::Error {
    fn from(value: DescriptorError) -> Self {
        io::Error::new(io::ErrorKind::InvalidData, value)
    }
}
impl Display for DescriptorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(match self {
            DescriptorError::Empty => "empty descriptor",
            DescriptorError::InfiniteClassName => "infinite class name",
            DescriptorError::UnknownType => "unknown type",
            DescriptorError::InvalidUtf8 => "invalid utf8",
            DescriptorError::NoArguments => "no method arguments",
            DescriptorError::InfiniteArguments => "infinite method arguments",
        }, f)
    }
}
impl Error for DescriptorError {}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldDescriptor {
    Byte, // B
    Char, // C
    Double, // D
    Float, // F
    Int, // I
    Long, // J
    ClassRef(Box<str>), // L s ;
    Short, // S
    Boolean, // Z
    ArrRef(Box<Self>), // [ _
}
impl FieldDescriptor {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DescriptorError> {
        Ok(match bytes.get(0).ok_or(DescriptorError::Empty)? {
            b'B' => Self::Byte,
            b'C' => Self::Char,
            b'D' => Self::Double,
            b'F' => Self::Float,
            b'I' => Self::Int,
            b'J' => Self::Long,
            b'S' => Self::Short,
            b'Z' => Self::Boolean,
            b'L' => Self::ClassRef({
                let Some(i) = bytes.iter().position(|&b| b == b';') else {
                    return Err(DescriptorError::InfiniteClassName);
                };
                String::from_utf8(bytes[1..i].to_vec())
                    .map_err(|_| DescriptorError::InvalidUtf8)?
                    .into_boxed_str()
            }),
            b'[' => Self::ArrRef(Box::new(Self::from_bytes(&bytes[1..])?)),
            _ => return Err(DescriptorError::UnknownType),
        })
    }
    fn length(&self) -> usize {
        match self {
            FieldDescriptor::Byte |
            FieldDescriptor::Char |
            FieldDescriptor::Double |
            FieldDescriptor::Float |
            FieldDescriptor::Int |
            FieldDescriptor::Long |
            FieldDescriptor::Short |
            FieldDescriptor::Boolean => 1,
            FieldDescriptor::ClassRef(s) => 2 + s.len(),
            FieldDescriptor::ArrRef(f) => 1 + f.length(),
        }
    }
    pub fn unit_size(&self) -> usize {
        match self {
            FieldDescriptor::Byte |
            FieldDescriptor::Char |
            FieldDescriptor::Float |
            FieldDescriptor::Int |
            FieldDescriptor::Short |
            FieldDescriptor::ClassRef(_) |
            FieldDescriptor::ArrRef(_) |
            FieldDescriptor::Boolean => 1,
            FieldDescriptor::Double |
            FieldDescriptor::Long => 2,
        }
    }
    pub fn display_type<'a>(&'a self) -> DisplayType<'a> {
        DisplayType(self)
    } 
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDescriptor {
    /// ( fd* )
    pub arg_types: Box<[FieldDescriptor]>,
    /// None is V (for void)
    pub return_type: Option<Box<FieldDescriptor>>,
}
impl MethodDescriptor {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, DescriptorError> {
        match bytes.get(0).ok_or(DescriptorError::Empty)? {
            b'(' => bytes = &bytes[1..],
            _ => return Err(DescriptorError::NoArguments),
        }
        let mut args = Vec::new();
        while *bytes.get(0).ok_or(DescriptorError::InfiniteArguments)? != b')' {
            match FieldDescriptor::from_bytes(bytes) {
                Ok(arg) => {
                    bytes = &bytes[arg.length()..];
                    args.push(arg);
                }
                Err(DescriptorError::Empty) => return Err(DescriptorError::InfiniteArguments),
                Err(e) => return Err(e),
            }
        }
        bytes = &bytes[1..];
        let return_type = match FieldDescriptor::from_bytes(bytes) {
            Ok(fd) => Some(Box::new(fd)),
            Err(DescriptorError::UnknownType) if bytes[0] == b'V' => None,
            Err(e) => return Err(e),
        };
        Ok(MethodDescriptor { arg_types: args.into_boxed_slice(), return_type })
    }
    pub fn display_type<'a>(&'a self, name: &'a str) -> DisplayMethodType<'a> {
        DisplayMethodType(name, self)
    } 
}
pub struct DisplayType<'a>(&'a FieldDescriptor);
impl Display for DisplayType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            FieldDescriptor::Byte => write!(f, "byte"),
            FieldDescriptor::Char => write!(f, "char"),
            FieldDescriptor::Double => write!(f, "double"),
            FieldDescriptor::Float => write!(f, "float"),
            FieldDescriptor::Int => write!(f, "int"),
            FieldDescriptor::Long => write!(f, "long"),
            FieldDescriptor::ClassRef(s) => write!(f, "{}", s.replace('/', ".")),
            FieldDescriptor::Short => write!(f, "short"),
            FieldDescriptor::Boolean => write!(f, "boolean"),
            FieldDescriptor::ArrRef(fd) => write!(f, "{}[]", fd.display_type()),
        }
    }
}
pub struct DisplayMethodType<'a>(&'a str, &'a MethodDescriptor);
impl Display for DisplayMethodType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.1.return_type {
            Some(t) => t.display_type().fmt(f)?,
            None => write!(f, "void")?,
        }
        write!(f, " {}(", self.0)?;
        let mut seperator = "";
        for arg in &self.1.arg_types {
            write!(f, "{seperator}{}", arg.display_type())?;
            seperator = ", ";
        }

        write!(f, ")")
    }
}
pub enum AnyDescriptor {
    Field(FieldDescriptor),
    Method(MethodDescriptor),
}
impl AnyDescriptor {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DescriptorError> {
        match MethodDescriptor::from_bytes(bytes) {
            Ok(md) => Ok(Self::Method(md)),
            Err(DescriptorError::NoArguments) => FieldDescriptor::from_bytes(bytes).map(Self::Field),
            Err(e) => Err(e),
        }
    }
    pub fn display_type<'a>(&'a self, name: &'a str) -> DisplayNamedAnyType<'a> {
        DisplayNamedAnyType(name, self)
    }
}
pub struct DisplayNamedAnyType<'a>(&'a str, &'a AnyDescriptor);
impl Display for DisplayNamedAnyType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            AnyDescriptor::Field(fd) => write!(f, "{} {}", fd.display_type(), self.0),
            AnyDescriptor::Method(md) => write!(f, "{}", md.display_type(self.0)),
        }
    }
}
