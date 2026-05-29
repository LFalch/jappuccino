use std::io::{self, Read, Write};

use crate::ReadIntExt;

pub fn read_modified_utf8<R: Read>(mut reader: R, length: usize) -> io::Result<String> {
    let mut string = String::with_capacity(length);
    let error = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            // TODO: perhaps give more specific errors in a custom type
            "incorrect \"modified\" utf-8 byte",
        )
    };
    let mut bytes = (0..length).map(|_| reader.read_u8());
    let mut high_surrogate = None;
    while let Some(b) = bytes.next() {
        let b = b?;
        // wtf is this
        match b {
            1..=0x7f => string.push(b as char),
            x @ 0b1100_0000..=0b1101_1111 => {
                let x = x as u32;
                let y = bytes.next().ok_or_else(error)?? as u32;
                let codepoint = (((x) & 0x1f) << 6) + ((y) & 0x3f);
                string.push(char::from_u32(codepoint).ok_or_else(error)?);
            }
            x @ 0b1110_0000..=0b1110_1111 => {
                let x = x as u32;
                let y = bytes.next().ok_or_else(error)?? as u32;
                let z = bytes.next().ok_or_else(error)?? as u32;
                let codepoint = ((x & 0xf) << 12) + ((y & 0x3f) << 6) + (z & 0x3f);
                if let Some(c) = char::from_u32(codepoint) {
                    string.push(c);
                } else {
                    match codepoint {
                        // high
                        0xd800..=0xdbff => {
                            if high_surrogate.is_some() {
                                return Err(error());
                            }
                            high_surrogate = Some(codepoint);
                            continue;
                        }
                        // low
                        0xdc00..=0xdfff => {
                            let low_surrogate = codepoint;
                            let high_surrogate = high_surrogate.take().ok_or_else(error)?;
                            let c = char::from_u32(
                                ((high_surrogate - 0xD800) << 10)
                                    + (low_surrogate - 0xDC00)
                                    + 0x1_0000,
                            )
                            .unwrap();
                            string.push(c);
                        }
                        _ => unreachable!(),
                    }
                }
            }
            _ => return Err(error()),
        }

        if high_surrogate.is_some() {
            // this is skipped if the high_surrogate was JUST set in this character
            return Err(error());
        }
    }

    Ok(string)
}

pub fn write_modified_utf8<W: Write>(writer: &mut W, s: &str) -> io::Result<()> {
    for c in s.chars() {
        let codepoint = c as u32;
        match codepoint {
            1..=0x7f => writer.write_all(&[codepoint as u8])?,
            0 | 0x80..=0x7ff => {
                let x = 0b1100_0000 | (codepoint >> 6) as u8;
                let y = 0b1000_0000 | (codepoint & 0b11_1111) as u8;
                writer.write_all(&[x, y])?;
            }
            0x800..=0xffff => {
                let x = 0b1110_0000 | (codepoint >> 12) as u8;
                let y = 0b1000_0000 | ((codepoint >> 6) & 0b11_1111) as u8;
                let z = 0b1000_0000 | (codepoint & 0b11_1111) as u8;
                writer.write_all(&[x, y, z])?;
            }
            0x10000.. => {
                let u = 0b1110_1101;
                let v = 0b1010_0000 | ((codepoint >> 16) - 1) as u8;
                let w = 0b1000_0000 | ((codepoint >> 10) & 0b11_1111) as u8;
                let x = 0b1110_1101;
                let y = 0b1011_0000 | ((codepoint >> 6) & 0b1111) as u8;
                let z = 0b1000_0000 | (codepoint & 0b11_1111) as u8;
                writer.write_all(&[u, v, w, x, y, z])?;
            }
        }
    }

    Ok(())
}
