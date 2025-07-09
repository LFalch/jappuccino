use std::fmt::{self, Display};

use num_enum::{TryFromPrimitive, FromPrimitive, IntoPrimitive};

use crate::class::{display_constant, Constant};

use self::opcode::Opcode;

pub mod opcode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code(pub Box<[u8]>);

pub struct DisplayCode<'a>{
    raw_code: &'a [u8],
    constant_pool: &'a [Constant],
    ident: usize,
}
pub fn display_code<'a>(raw_code: &'a [u8], constant_pool: &'a [Constant], ident: usize) -> DisplayCode<'a> {
    DisplayCode {
        raw_code,
        constant_pool,
        ident,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum PrimitiveArrayType {
    Boolean = 4,
    Char = 5,
    Float = 6,
    Double = 7,
    Byte = 8,
    Short = 9,
    Int = 10,
    Long = 11,
}

impl Display for DisplayCode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.raw_code.is_empty() { return Ok(()) }

        let mut bytes = self.raw_code.iter().copied().enumerate();
        let indent = self.ident;
        // :( allocation
        let indent = " ".repeat(indent);
        let pc_width_max = (self.raw_code.len().ilog10() + 1) as usize;

        while let Some((i, opcode)) = bytes.next() {
            let opcode = Opcode::from_primitive(opcode);
            write!(f, "{indent}{i: >pc_width_max$}: {}", opcode.mnemonic())?;
            display_arg(opcode, f, &mut bytes, i, self.constant_pool)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

fn extract_u16(bytes: &mut impl Iterator<Item=(usize, u8)>) -> Option<(usize, u16)> {
    let (n, b1) = bytes.next()?;
    let (_, b2) = bytes.next()?;
    Some((n, u16::from_be_bytes([b1, b2])))
}
fn extract_u32(bytes: &mut impl Iterator<Item=(usize, u8)>) -> Option<(usize, u32)> {
    let (n, b1) = bytes.next()?;
    let (_, b2) = bytes.next()?;
    let (_, b3) = bytes.next()?;
    let (_, b4) = bytes.next()?;
    Some((n, u32::from_be_bytes([b1, b2, b3, b4])))
}
fn display_arg_byte(f: &mut fmt::Formatter<'_>, bytes: &mut impl Iterator<Item=(usize, u8)>) -> fmt::Result {
    let Some((_, byte)) = bytes.next() else {
        return Ok(());
    };
    write!(f, " {byte}")
}
fn display_arg_short(f: &mut fmt::Formatter<'_>, bytes: &mut impl Iterator<Item=(usize, u8)>) -> fmt::Result {
    let Some((_, short)) = extract_u16(bytes) else {
        return Ok(());
    };
    write!(f, " {short}")
}
fn display_arg_const_index(f: &mut fmt::Formatter<'_>, bytes: &mut impl Iterator<Item=(usize, u8)>, constant_pool: &[Constant]) -> fmt::Result {
    let Some((_, index)) = extract_u16(bytes) else {
        return Ok(());
    };
    write!(f, " #{index}")?;
    display_const_as_comment(f, index, constant_pool)
}
fn display_arg_branch_offset(f: &mut fmt::Formatter<'_>, bytes: &mut impl Iterator<Item=(usize, u8)>) -> fmt::Result {
    let Some((s, offset)) = extract_u16(bytes) else {
        return Ok(());
    };
    let target = (s as u32 - 1).wrapping_add(offset as i16 as i32 as u32);
    write!(f, " .{target}")
}
fn display_arg_branch_offset_w(f: &mut fmt::Formatter<'_>, bytes: &mut impl Iterator<Item=(usize, u8)>) -> fmt::Result {
    let Some((s, offset)) = extract_u32(bytes) else {
        return Ok(());
    };
    let target = (s as u32 - 1).wrapping_add(offset);
    write!(f, " .{target}")
}
fn display_arg_const_index_byte(f: &mut fmt::Formatter<'_>, bytes: &mut impl Iterator<Item=(usize, u8)>, constant_pool: &[Constant]) -> fmt::Result {
    let Some((_, index)) = bytes.next() else {
        return Ok(());
    };
    write!(f, " #{index}")?;
    display_const_as_comment(f, index as u16, constant_pool)
}
fn display_const_as_comment(f: &mut fmt::Formatter<'_>, index: u16, constant_pool: &[Constant]) -> fmt::Result {
    write!(f, " \t// {}", display_constant(index as u16, constant_pool))
}
fn display_arg(opcode: Opcode, f: &mut fmt::Formatter<'_>, bytes: &mut impl Iterator<Item=(usize, u8)>, i: usize, constant_pool: &[Constant]) -> fmt::Result {
    use Opcode::*;
    match opcode {
        Nop |
        AconstNull |
        IconstM1 |
        Iconst0 |
        Iconst1 |
        Iconst2 |
        Iconst3 |
        Iconst4 |
        Iconst5 |
        Lconst0 |
        Lconst1 |
        Fconst0 |
        Fconst1 |
        Fconst2 |
        Dconst0 |
        Dconst1 |
        Iload0 |
        Iload1 |
        Iload2 |
        Iload3 |
        Lload0 |
        Lload1 |
        Lload2 |
        Lload3 |
        Fload0 |
        Fload1 |
        Fload2 |
        Fload3 |
        Dload0 |
        Dload1 |
        Dload2 |
        Dload3 |
        Aload0 |
        Aload1 |
        Aload2 |
        Aload3 |
        Iaload |
        Laload |
        Faload |
        Daload |
        Aaload |
        Baload |
        Caload |
        Saload |
        Istore0 |
        Istore1 |
        Istore2 |
        Istore3 |
        Lstore0 |
        Lstore1 |
        Lstore2 |
        Lstore3 |
        Fstore0 |
        Fstore1 |
        Fstore2 |
        Fstore3 |
        Dstore0 |
        Dstore1 |
        Dstore2 |
        Dstore3 |
        Astore0 |
        Astore1 |
        Astore2 |
        Astore3 |
        Iastore |
        Lastore |
        Fastore |
        Dastore |
        Aastore |
        Bastore |
        Castore |
        Sastore |
        Pop |
        Pop2 |
        Dup |
        DupX1 |
        DupX2 |
        Dup2 |
        Dup2X1 |
        Dup2X2 |
        Swap |
        Iadd |
        Ladd |
        Fadd |
        Dadd |
        Isub |
        Lsub |
        Fsub |
        Dsub |
        Imul |
        Lmul |
        Fmul |
        Dmul |
        Idiv |
        Ldiv |
        Fdiv |
        Ddiv |
        Irem |
        Lrem |
        Frem |
        Drem |
        Ineg |
        Lneg |
        Fneg |
        Dneg |
        Ishl |
        Lshl |
        Ishr |
        Lshr |
        Iushr |
        Lushr |
        Iand |
        Land |
        Ior |
        Lor |
        Ixor |
        Lxor |
        I2l |
        I2f |
        I2d |
        L2i |
        L2f |
        L2d |
        F2i |
        F2l |
        F2d |
        D2i |
        D2l |
        D2f |
        I2b |
        I2c |
        I2s |
        Lcmp |
        Fcmpl |
        Fcmpg |
        Dcmpl |
        Dcmpg |
        Ireturn |
        Lreturn |
        Freturn |
        Dreturn |
        Areturn |
        Return |
        Arraylength |
        Athrow |
        Monitorenter |
        Monitorexit => (),
        Anewarray |
        Checkcast |
        Instanceof |
        New | Invokespecial | Invokestatic | Invokevirtual | Getfield | Getstatic | Putfield | Putstatic | LdcW | Ldc2W => display_arg_const_index(f, bytes, constant_pool)?,
        Invokeinterface | Invokedynamic => {
            let Some((_, ih)) = bytes.next() else {
                return Ok(());
            };
            let Some((_, il)) = bytes.next() else {
                return Ok(());
            };
            let index = u16::from_be_bytes([ih, il]);
            let Some((_, h)) = bytes.next() else {
                return Ok(());
            };
            let Some((_, l)) = bytes.next() else {
                return Ok(());
            };
            let zero = u16::from_be_bytes([h, l]);
            write!(f, " #{index}")?;
            if zero != 0 {
                write!(f, ", {zero}")?;
            }
            display_const_as_comment(f, index, constant_pool)?;
        }
        Ldc => display_arg_const_index_byte(f, bytes, constant_pool)?,
        Aload |
        Astore |
        Dload |
        Dstore |
        Fload |
        Fstore |
        Iload |
        Istore |
        Lload |
        Lstore |
        Ret |
        Bipush => display_arg_byte(f, bytes)?,
        Sipush => display_arg_short(f, bytes)?,
        Iinc => {
            let Some((_, i)) = bytes.next() else {
                return Ok(());
            };
            let Some((_, c)) = bytes.next() else {
                return Ok(());
            };
            write!(f, " {i}, {c}")?;
        }
        Ifeq |
        Ifne |
        Iflt |
        Ifge |
        Ifgt |
        Ifle |
        IfIcmpeq |
        IfIcmpne |
        IfIcmplt |
        IfIcmpge |
        IfIcmpgt |
        IfIcmple |
        IfAcmpeq |
        IfAcmpne |
        Goto |
        Jsr |
        Ifnull |
        Ifnonnull => display_arg_branch_offset(f, bytes)?,
        GotoW |
        JsrW => display_arg_branch_offset_w(f, bytes)?,
        Tableswitch => {
            let padding = (((i + 1) ^ 3) + 1) & 3;
            let mut sep = "";
            write!(f, "{{")?;
            for _ in 0..padding {
                let Some((_, b)) = bytes.next() else {
                    return Ok(())
                };
                write!(f, "{sep}{b}")?;
                sep = ", ";
            }
            write!(f, "}}")?;
            let Some((_, default)) = extract_u32(bytes) else {
                return Ok(())
            };
            let default = default as i32;
            let Some((_, low)) = extract_u32(bytes) else {
                return Ok(())
            };
            let low = low as i32;
            let Some((_, high)) = extract_u32(bytes) else {
                return Ok(())
            };
            let high = high as i32;
            for index in low..=high {
                let Some((_, offset)) = extract_u32(bytes) else {
                    return Ok(())
                };
                write!(f, ", {index}: {}", i.wrapping_add(offset as i32 as i64 as u64 as usize))?;
            }
            write!(f, ", default: {}", i.wrapping_add(default as i64 as u64 as usize))?;
        }
        Lookupswitch => {
            let padding = (((i + 1) ^ 3) + 1) & 3;
            let mut sep = "";
            write!(f, "{{")?;
            for _ in 0..padding {
                let Some((_, b)) = bytes.next() else {
                    return Ok(())
                };
                write!(f, "{sep}{b}")?;
                sep = ", ";
            }
            write!(f, "}}")?;
            let Some((_, default)) = extract_u32(bytes) else {
                return Ok(())
            };
            let default = default as i32;
            let Some((_, npairs)) = extract_u32(bytes) else {
                return Ok(())
            };
            for _ in 0..npairs {
                let Some((_, match_key)) = extract_u32(bytes) else {
                    return Ok(())
                };
                let match_key = match_key as i32;
                let Some((_, offset)) = extract_u32(bytes) else {
                    return Ok(())
                };
                write!(f, ", {match_key}: {}", i.wrapping_add(offset as i32 as i64 as u64 as usize))?;
            }
            write!(f, ", default: {}", i.wrapping_add(default as i64 as u64 as usize))?;
        }
        Multianewarray => {
            let Some((_, index)) = extract_u16(bytes) else {
                return Ok(());
            };
            let Some((_, dimensions)) = bytes.next() else {
                return Ok(());
            };
            write!(f, " #{index}, {dimensions}")?;
        }
        Newarray => {
            let Some((_, t)) = bytes.next() else {
                return Ok(());
            };
            match PrimitiveArrayType::try_from_primitive(t) {
                Ok(PrimitiveArrayType::Boolean) => write!(f, " boolean")?,
                Ok(PrimitiveArrayType::Char) => write!(f, " char")?,
                Ok(PrimitiveArrayType::Float) => write!(f, " float")?,
                Ok(PrimitiveArrayType::Double) => write!(f, " double")?,
                Ok(PrimitiveArrayType::Byte) => write!(f, " byte")?,
                Ok(PrimitiveArrayType::Short) => write!(f, " short")?,
                Ok(PrimitiveArrayType::Int) => write!(f, " int")?,
                Ok(PrimitiveArrayType::Long) => write!(f, " long")?,
                Err(_) => write!(f, " ??{t}")?,
            }
        }
        Wide => {
            let Some((_, o)) = bytes.next() else {
                return Ok(());
            };
            let o = Opcode::from_primitive(o);
            match o {
                Iload |
                Fload |
                Aload |
                Lload |
                Dload |
                Istore |
                Fstore |
                Astore |
                Lstore |
                Dstore |
                Ret => {
                    write!(f, " {}", o.mnemonic())?;
                    display_arg_short(f, bytes)?;
                },
                Iinc => {
                    write!(f, " {}", o.mnemonic())?;
                    display_arg_short(f, bytes)?;
                    write!(f, ",")?;
                    display_arg_short(f, bytes)?;
                }
                _ => write!(f, "???")?,
            }
        }
        Impdep1 |
        Impdep2 |
        Breakpoint |
        ReservedFuture => (),
    }
    Ok(())
}