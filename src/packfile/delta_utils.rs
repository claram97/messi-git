use std::io::{self, Read};

const COPY_INSTRUCTION_FLAG: u8 = 1 << 7;
const COPY_OFFSET_BYTES: u8 = 4;
const COPY_SIZE_BYTES: u8 = 3;
const COPY_ZERO_SIZE: usize = 0x10000;
const MAX_COPY_SIZE: usize = 0x7F;

// Read an integer of up to `bytes` bytes.
// `present_bytes` indicates which bytes are provided. The others are 0.
fn read_partial_int<R: Read>(
    stream: &mut R,
    bytes: u8,
    present_bytes: &mut u8,
) -> io::Result<usize> {
    let mut value = 0;
    for byte_index in 0..bytes {
        // Use one bit of `present_bytes` to determine if the byte exists
        if *present_bytes & 1 != 0 {
            let [byte] = read_bytes(stream)?;
            value |= (byte as usize) << (byte_index * 8);
        }
        *present_bytes >>= 1;
    }
    Ok(value)
}

// Reads a single delta instruction from a stream
// and appends the relevant bytes to `result`.
// Returns whether the delta stream still had instructions.
pub fn apply_delta_instruction<R: Read>(
    stream: &mut R,
    base: &[u8],
    result: &mut Vec<u8>,
) -> io::Result<bool> {
    // Check if the stream has ended, meaning the new object is done
    let instruction = match read_bytes(stream) {
        Ok([instruction]) => instruction,
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(false),
        Err(err) => return Err(err),
    };
    if instruction & COPY_INSTRUCTION_FLAG == 0 {
        // Data instruction; the instruction byte specifies the number of data bytes
        if instruction == 0 {
            // Appending 0 bytes doesn't make sense, so git disallows it
            return Err(make_error("Invalid data instruction"));
        }

        // Append the provided bytes
        let mut data = vec![0; instruction as usize];
        stream.read_exact(&mut data)?;
        result.extend_from_slice(&data);
    } else {
        // Copy instruction
        let mut nonzero_bytes = instruction;
        let offset = read_partial_int(stream, COPY_OFFSET_BYTES, &mut nonzero_bytes)?;
        let mut size = read_partial_int(stream, COPY_SIZE_BYTES, &mut nonzero_bytes)?;
        if size == 0 {
            // Copying 0 bytes doesn't make sense, so git assumes a different size
            size = COPY_ZERO_SIZE;
        }
        // Copy bytes from the base object
        let base_data = base
            .get(offset..(offset + size))
            .ok_or_else(|| make_error("Invalid copy instruction"))?;
        result.extend_from_slice(base_data);
    }
    Ok(true)
}

pub fn make_error(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_string())
}

fn read_bytes<R: Read, const N: usize>(stream: &mut R) -> io::Result<[u8; N]> {
    let mut bytes = [0; N];
    stream.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub fn read_size_encoding<R: Read>(stream: &mut R) -> io::Result<usize> {
    let size_bytes = read_encoding_bytes(stream)?;
    Ok(decode_size(&size_bytes))
}

pub fn read_offset_encoding<R: Read>(stream: &mut R) -> io::Result<u64> {
    let offset_bytes = read_encoding_bytes(stream)?;
    Ok(decode_offset(&offset_bytes))
}

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

pub fn encode_offset(n: usize) -> Vec<u8> {
    let mut encoded = Vec::new();
    let mut n = n + 1;
    while n > 0 {
        n -= 1;
        encoded.push(((n as u8) & 0x7F) | 0x80);
        n >>= 7;
    }
    encoded[0] = encoded[0] & 0x7F;
    encoded.reverse();
    encoded
}

fn decode_offset(bytes: &[u8]) -> u64 {
    let mut value = 0;
    for byte in bytes {
        value = value << 7 | (byte & 0x7F) as u64;
        value += 1;
    }
    value - 1
}

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

fn decode_size(bytes: &[u8]) -> usize {
    let mut n = 0;
    for (i, byte) in bytes.iter().enumerate() {
        n |= ((byte & 0x7F) as usize) << (i * 7);
    }
    n
}

#[derive(Debug)]
pub enum Command {
    Copy { offset: usize, size: usize },
    Insert(Vec<u8>),
}

impl Command {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Command::Copy { offset, size } => {
                let mut encoded = Vec::new();
                let offset = encode_size(*offset);
                let size = encode_size(*size);
                encoded.extend_from_slice(&offset);
                encoded.extend_from_slice(&size);
                encoded
            }
            Command::Insert(bytes) => {
                let mut encoded = Vec::new();
                bytes.chunks(MAX_COPY_SIZE).for_each(|chunk| {
                    let header = 0x7F & chunk.len() as u8;
                    encoded.push(header);
                    encoded.extend_from_slice(bytes);
                });
                encoded
            }
        }
    }
}

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
    commands
}

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
        
        assert_eq!(encode_offset(479), vec![130,95]);
        assert_eq!(decode_offset(&[130,95]), 479);
        
        assert_eq!(encode_offset(499), vec![130,115]);
        assert_eq!(decode_offset(&[130,115]), 499);
        
        assert_eq!(encode_offset(446), vec![130,62]);
        assert_eq!(decode_offset(&[130,62]), 446);
        
        assert_eq!(encode_offset(566), vec![131,54]);
        assert_eq!(decode_offset(&[131,54]), 566);
        
        assert_eq!(encode_offset(584), vec![131,72]);
        assert_eq!(decode_offset(&[131,72]), 584);
        
        assert_eq!(encode_offset(138), vec![128,10]);
        assert_eq!(decode_offset(&[128,10]), 138);
        
        assert_eq!(encode_offset(717), vec![132,77]);
        assert_eq!(decode_offset(&[132,77]), 717);
        
        assert_eq!(encode_offset(812), vec![133,44]);
        assert_eq!(decode_offset(&[133,44]), 812);
        
        assert_eq!(encode_offset(1187), vec![136,35]);
        assert_eq!(decode_offset(&[136,35]), 1187);
        
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
}
