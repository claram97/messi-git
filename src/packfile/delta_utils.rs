use std::io::{Read, self};

const COPY_INSTRUCTION_FLAG: u8 = 1 << 7;
const COPY_OFFSET_BYTES: u8 = 4;
const COPY_SIZE_BYTES: u8 = 3;
const COPY_ZERO_SIZE: usize = 0x10000;

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

// Read 7 bits of data and a flag indicating whether there are more
fn read_varint_byte<R: Read>(stream: &mut R) -> io::Result<(u8, bool)> {
    let [byte] = read_bytes(stream)?;
    let value = byte & 0x7f;
    let more_bytes = byte & 0x80 != 0;
    Ok((value, more_bytes))
}

pub fn read_size_encoding<R: Read>(stream: &mut R) -> io::Result<usize> {
    let mut value = 0;
    let mut length = 0; // the number of bits of data read so far
    loop {
        let (byte_value, more_bytes) = read_varint_byte(stream)?;
        // Add in the data bits
        value |= (byte_value as usize) << length;
        // Stop if this is the last byte
        if !more_bytes {
            return Ok(value);
        }

        length += 7;
    }
}

pub fn encode_size(n: usize) -> Vec<u8> {
    let mut n = n;
    let mut encoded_size = Vec::new();
    while n > 0 {
        let m = (n as u8) & 0x7f;
        n >>= 7;
        if n > 0 {
            encoded_size.push(0x80 | m)
        } else {
            encoded_size.push(m)
        }
    }
    return encoded_size;
}
