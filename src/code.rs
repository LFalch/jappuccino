use std::fmt::{self, Display};

use num_enum::FromPrimitive;

use self::opcode::Opcode;

pub mod opcode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code(pub Box<[u8]>);

impl Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() { return Ok(()) }

        let mut bytes = self.0.iter().copied().enumerate();
        let indent = f.precision().unwrap_or(0);
        // :( allocation
        let indent = " ".repeat(indent);
        let pc_width_max = (self.0.len().ilog10() + 1) as usize;

        while let Some((i, opcode)) = bytes.next() {
            let opcode = Opcode::from_primitive(opcode);
            write!(f, "{indent}{i: >pc_width_max$}: {}", opcode.mnemonic())?;
            let mut separator = " ";
            // TODO: for relevant instructions, look up meaning of values from the constant pool
            for b in (0..opcode.immediates()).map_while(|_| bytes.next().map(|p| p.1)) {
                write!(f, "{separator}{b}")?;
                separator = ", ";
            }
            writeln!(f)?;
        }

        Ok(())
    }
}
