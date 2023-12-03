use std::io::{self, Read};

const COPY_COMMAND_FLAG: u8 = 0x80;
const COPY_OFFSET_BYTES: u8 = 4;
const COPY_SIZE_BYTES: u8 = 3;
const COPY_ZERO_SIZE: usize = 0x10000;
const MAX_INSERT_SIZE: usize = 0x7F;
const MAX_COPY_SIZE: usize = 0xFFFFFF;

// Helper function to read offset and size from a delta copy instruction
// The offset and size are encoded in the first byte of the instruction
// The offset is encoded in the first 4 bits
// The size is encoded in the last 3 bits
// The offset and size are encoded in little endian
fn read_encoded_copy_int<R: Read>(
    stream: &mut R,
    bytes: u8,
    encoded_bits: u8,
) -> io::Result<usize> {
    let mut value = 0;
    let mut encoded_bits = encoded_bits;
    for byte_index in 0..bytes {
        // Use one bit of `encoded_bits` to determine if the byte exists
        if encoded_bits & 1 != 0 {
            let [byte] = read_bytes(stream)?;
            value |= (byte as usize) << (byte_index * 8);
        }
        encoded_bits >>= 1;
    }
    Ok(value)
}

/// Read a sequence of delta commands from a packfile
/// The sequence ends when an EOF is reached
///
/// # Arguments
///
/// * `packfile` - A mutable reference to a packfile
///
/// # Returns
///
/// A vector of commands
pub fn read_delta_commands<R: Read>(packfile: &mut R) -> io::Result<Vec<Command>> {
    let mut commands = Vec::new();
    loop {
        let command = match read_bytes(packfile) {
            Ok([command]) => command,
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(commands),
            Err(err) => return Err(err),
        };
        if command & COPY_COMMAND_FLAG == 0 {
            if command == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid data command",
                ));
            }
            let mut data = vec![0; command as usize];
            packfile.read_exact(&mut data)?;
            commands.push(Command::Insert(data));
        } else {
            let offset = read_encoded_copy_int(packfile, COPY_OFFSET_BYTES, command)?;
            let mut size =
                read_encoded_copy_int(packfile, COPY_SIZE_BYTES, command >> COPY_OFFSET_BYTES)?;
            if size == 0 {
                size = COPY_ZERO_SIZE;
            }
            commands.push(Command::Copy { offset, size });
        }
    }
}

// Helper function to read exactly N bytes from a stream
fn read_bytes<R: Read, const N: usize>(stream: &mut R) -> io::Result<[u8; N]> {
    let mut bytes = [0; N];
    stream.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Read an usize from a stream encoded with the variable length encoding
///
/// # Arguments
///
/// * `stream` - A mutable reference to a stream
///
/// # Returns
///
/// An usize
pub fn read_size_encoding<R: Read>(stream: &mut R) -> io::Result<usize> {
    let size_bytes = read_encoding_bytes(stream)?;
    Ok(decode_size(&size_bytes))
}

/// Read an usize from a stream encoded with the offset encoding
///
/// # Arguments
///
/// * `stream` - A mutable reference to a stream
///
/// # Returns
///
/// An usize
pub fn read_offset_encoding<R: Read>(stream: &mut R) -> io::Result<u64> {
    let offset_bytes = read_encoding_bytes(stream)?;
    Ok(decode_offset(&offset_bytes))
}

// Helper function to read a variable length encoding from a stream
fn read_encoding_bytes<R: Read>(stream: &mut R) -> io::Result<Vec<u8>> {
    let mut size_bytes = Vec::new();
    loop {
        let [byte] = read_bytes(stream)?;
        size_bytes.push(byte);
        if byte & 0x80 == 0 {
            break;
        }
    }
    Ok(size_bytes)
}

/// Encode an usize with the offset encoding
///
/// # Arguments
///
/// * `n` - An usize
///
/// # Returns
///
/// A vector of bytes
pub fn encode_offset(n: usize) -> Vec<u8> {
    let mut encoded = Vec::new();
    let mut n = n + 1;
    while n > 0 {
        n -= 1;
        encoded.push(((n as u8) & 0x7F) | 0x80);
        n >>= 7;
    }
    encoded[0] &= 0x7F;
    encoded.reverse();
    encoded
}

// Helper function to decode an offset from a vector of bytes
fn decode_offset(bytes: &[u8]) -> u64 {
    let mut value = 0;
    for byte in bytes {
        value = value << 7 | (byte & 0x7F) as u64;
        value += 1;
    }
    value - 1
}

/// Encode an usize with the variable length encoding
///
/// # Arguments
///
/// * `n` - An usize
///
/// # Returns
///
/// A vector of bytes
pub fn encode_size(n: usize) -> Vec<u8> {
    let mut n = n;
    let mut encoded_size = Vec::new();
    while n >= 128 {
        encoded_size.push(((n as u8) & 0x7F) | 0x80);
        n >>= 7;
    }
    encoded_size.push(n as u8);
    encoded_size
}

// Helper function to decode an usize from a vector of bytes
fn decode_size(bytes: &[u8]) -> usize {
    let mut n = 0;
    for (i, byte) in bytes.iter().enumerate() {
        n |= ((byte & 0x7F) as usize) << (i * 7);
    }
    n
}

/// A delta command
///  
/// # Variants
///
/// * `Copy` - A copy command
/// * `Insert` - An insert command
#[derive(Debug, Clone)]
pub enum Command {
    Copy { offset: usize, size: usize },
    Insert(Vec<u8>),
}

impl Command {
    /// Encode a command into a vector of bytes
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Command::Copy { offset, size } => Self::encode_copy(*offset, *size),
            Command::Insert(bytes) => Self::encode_insert(bytes),
        }
    }

    fn encode_copy(offset: usize, size: usize) -> Vec<u8> {
        if size > MAX_COPY_SIZE {
            let size_1 = MAX_COPY_SIZE;
            let size_2 = size - MAX_COPY_SIZE;
            let offset_2 = offset + MAX_COPY_SIZE;
            let mut encoded = Self::encode_copy(offset, size_1);
            let mut encoded_2 = Self::encode_copy(offset_2, size_2);
            encoded.append(&mut encoded_2);
            return encoded;
        }
        let mut encoded = vec![0x80];
        let offset = offset.to_le_bytes();
        let size = size.to_le_bytes();

        for (i, byte) in offset.iter().enumerate().take(4) {
            if offset[i] != 0 {
                encoded.push(*byte);
                encoded[0] |= 1 << i;
            }
        }
        for (i, byte) in size.iter().enumerate().take(3) {
            if size[i] != 0 {
                encoded.push(*byte);
                encoded[0] |= 1 << (i + 4);
            }
        }
        encoded
    }

    fn encode_insert(bytes: &[u8]) -> Vec<u8> {
        let mut encoded = Vec::new();
        bytes.chunks(MAX_INSERT_SIZE).for_each(|chunk| {
            let header = 0x7F & chunk.len() as u8;
            encoded.push(header);
            encoded.extend_from_slice(chunk);
        });
        encoded
    }
}

/// Create a sequence of delta commands from two objects
///
/// # Arguments
///
/// * `base` - The base object
/// * `object` - The object to be created
///
/// # Returns
///
/// A vector of commands that can be used to recreate the object from the base
pub fn delta_commands_from_objects(base: &[u8], object: &[u8]) -> Vec<Command> {
    let blines = base.split_inclusive(|&c| c == b'\n').collect::<Vec<_>>();
    let olines = object.split_inclusive(|&c| c == b'\n').collect::<Vec<_>>();
    let mut commands = Vec::new();

    let mut base_lines_read = 0;
    let mut last_offset = 0;

    for oline in olines {
        let mut offset = 0;
        let mut lines_read = 0;
        let copy = blines.iter().skip(base_lines_read).any(|&bline| {
            lines_read += 1;
            offset += bline.len();
            bline == oline
        });
        let size = oline.len();
        if copy {
            base_lines_read += lines_read;
            last_offset += offset;
            commands.push(Command::Copy {
                offset: last_offset - size,
                size,
            });
        } else {
            commands.push(Command::Insert(oline.to_vec()));
        }
    }
    optimize_delta_commands(commands.as_slice())
}

// Helper function to optimize a sequence of delta commands
// This function merges consecutive copy commands and consecutive insert commands
fn optimize_delta_commands(commands: &[Command]) -> Vec<Command> {
    let mut optimized = vec![commands[0].clone()];
    let mut i = 0;
    for command in &commands[1..] {
        match (&optimized[i], command) {
            (
                Command::Copy {
                    offset: o1,
                    size: s1,
                },
                Command::Copy {
                    offset: o2,
                    size: s2,
                },
            ) => {
                if *o1 + *s1 == *o2 {
                    optimized[i] = Command::Copy {
                        offset: *o1,
                        size: *s1 + *s2,
                    };
                } else {
                    optimized.push(command.clone());
                    i += 1;
                }
            }
            (Command::Insert(b1), Command::Insert(b2)) => {
                optimized[i] = Command::Insert([b1.as_slice(), b2.as_slice()].concat());
            }
            _ => {
                optimized.push(command.clone());
                i += 1;
            }
        }
    }
    optimized
}

/// Recreate an object from a base and a sequence of commands
///
/// # Arguments
///
/// * `base` - The base object
/// * `commands` - The sequence of commands
///
/// # Returns
///
/// A vector of bytes representing the recreated object
pub fn recreate_from_commands(base: &[u8], commands: &[Command]) -> Vec<u8> {
    let mut recreated = Vec::new();
    for c in commands {
        match c {
            Command::Copy { offset, size } => {
                let copied = &base[*offset..offset + size];
                recreated.extend_from_slice(copied);
            }
            Command::Insert(bytes) => {
                recreated.extend_from_slice(bytes);
            }
        }
    }
    recreated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_size() {
        assert_eq!(encode_size(0), vec![0]);
        assert_eq!(decode_size(&[0]), 0);

        assert_eq!(encode_size(1), vec![1]);
        assert_eq!(decode_size(&[1]), 1);

        assert_eq!(encode_size(127), vec![127]);
        assert_eq!(decode_size(&[127]), 127);

        assert_eq!(encode_size(128), vec![128, 1]);
        assert_eq!(decode_size(&[128, 1]), 128);

        assert_eq!(encode_size(129), vec![129, 1]);
        assert_eq!(decode_size(&[129, 1]), 129);

        assert_eq!(encode_size(206), vec![206, 1]);
        assert_eq!(decode_size(&[206, 1]), 206);

        assert_eq!(encode_size(255), vec![255, 1]);
        assert_eq!(decode_size(&[255, 1]), 255);

        assert_eq!(encode_size(256), vec![128, 2]);
        assert_eq!(decode_size(&[128, 2]), 256);

        assert_eq!(encode_size(257), vec![129, 2]);
        assert_eq!(decode_size(&[129, 2]), 257);

        assert_eq!(encode_size(16383), vec![255, 127]);
        assert_eq!(decode_size(&[255, 127]), 16383);

        assert_eq!(encode_size(16384), vec![128, 128, 1]);
        assert_eq!(decode_size(&[128, 128, 1]), 16384);

        assert_eq!(encode_size(16385), vec![129, 128, 1]);
        assert_eq!(decode_size(&[129, 128, 1]), 16385);

        assert_eq!(encode_size(2097151), vec![255, 255, 127]);
        assert_eq!(decode_size(&[255, 255, 127]), 2097151);

        assert_eq!(encode_size(2097152), vec![128, 128, 128, 1]);
        assert_eq!(decode_size(&[128, 128, 128, 1]), 2097152);

        assert_eq!(encode_size(2097153), vec![129, 128, 128, 1]);
        assert_eq!(decode_size(&[129, 128, 128, 1]), 2097153);

        assert_eq!(encode_size(268435455), vec![255, 255, 255, 127]);
        assert_eq!(decode_size(&[255, 255, 255, 127]), 268435455);
    }

    #[test]
    fn test_encode_decode_offset() {
        assert_eq!(encode_offset(53), vec![53]);
        assert_eq!(decode_offset(&[53]), 53);

        assert_eq!(encode_offset(79), vec![79]);
        assert_eq!(decode_offset(&[79]), 79);

        assert_eq!(encode_offset(111), vec![111]);
        assert_eq!(decode_offset(&[111]), 111);

        assert_eq!(encode_offset(479), vec![130, 95]);
        assert_eq!(decode_offset(&[130, 95]), 479);

        assert_eq!(encode_offset(499), vec![130, 115]);
        assert_eq!(decode_offset(&[130, 115]), 499);

        assert_eq!(encode_offset(446), vec![130, 62]);
        assert_eq!(decode_offset(&[130, 62]), 446);

        assert_eq!(encode_offset(566), vec![131, 54]);
        assert_eq!(decode_offset(&[131, 54]), 566);

        assert_eq!(encode_offset(584), vec![131, 72]);
        assert_eq!(decode_offset(&[131, 72]), 584);

        assert_eq!(encode_offset(138), vec![128, 10]);
        assert_eq!(decode_offset(&[128, 10]), 138);

        assert_eq!(encode_offset(717), vec![132, 77]);
        assert_eq!(decode_offset(&[132, 77]), 717);

        assert_eq!(encode_offset(812), vec![133, 44]);
        assert_eq!(decode_offset(&[133, 44]), 812);

        assert_eq!(encode_offset(1187), vec![136, 35]);
        assert_eq!(decode_offset(&[136, 35]), 1187);
    }

    #[test]
    fn test_recreate_from_commands() -> io::Result<()> {
        let base = "let mode = String::from_utf8(mode.to_vec())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?; // lo paso a string
    let hash: Vec<String> = hash.iter().map(|byte| format!(\"{:02x}\", byte)).collect(); // convierto los bytes del hash a string
    let hash = hash.concat().to_string();
    let name = String::from_utf8(name.to_vec())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    results.push((mode, name, hash)); // agrego el resultado y vuelvo a empezar".as_bytes();
        let object = "let mode = String::from_utf8(mode.to_vec())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?; // lo paso a string
    let hash: Vec<String> = hash.iter().map(|byte| format!(\"{:02x}\", byte)).collect(); // convierto los bytes del hash a string
    let hash = hash.concat().to_string();
    // un comentario en el medio
    let name = String::from_utf8(name.to_vec())
    // una linea de comentarios mas
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    results.push((mode, name, hash)); // agrego el resultado y vuelvo a empezar
    // termino con un comentario
    ".as_bytes();
        let commands = delta_commands_from_objects(base, object);
        let recreated = recreate_from_commands(base, &commands);
        assert_eq!(recreated.len(), object.len());
        assert_eq!(recreated, object);
        Ok(())
    }

    #[test]
    fn test_copy_encoding() {
        let command = Command::Copy {
            offset: 0x00,
            size: 0x56,
        };
        let encoded = command.encode();
        assert_eq!(encoded, vec![0b10010000, 0x56]);

        let command = Command::Copy {
            offset: 0x12,
            size: 0x56,
        };
        let encoded = command.encode();
        assert_eq!(encoded, vec![0b10010001, 0x12, 0x56]);

        let command = Command::Copy {
            offset: 0x1234,
            size: 0x5678,
        };
        let encoded = command.encode();
        assert_eq!(encoded, vec![0b10110011, 0x34, 0x12, 0x78, 0x56]);

        let command = Command::Copy {
            offset: 0x1234A5,
            size: 0x5678B4,
        };
        let encoded = command.encode();
        assert_eq!(
            encoded,
            vec![0b11110111, 0xA5, 0x34, 0x12, 0xB4, 0x78, 0x56]
        );

        let command = Command::Copy {
            offset: 0xA0B1C2D3,
            size: 0x5678D8,
        };
        let encoded = command.encode();
        assert_eq!(
            encoded,
            vec![0b11111111, 0xD3, 0xC2, 0xB1, 0xA0, 0xD8, 0x78, 0x56]
        );
    }
}
