use std::io;
use std::io::ErrorKind;
use std::mem;

pub fn read_u8(reader: &mut io::Read) -> io::Result<u8> {
    let mut buffer = [0 as u8; 1];
    try!(reader.read_exact(&mut buffer));

    Ok(buffer[0])
}

pub fn read_u32(reader: &mut io::Read) -> io::Result<u32> {
    let mut buffer = [0 as u8; 4];
    try!(reader.read_exact(&mut buffer));

    let value: u32 = unsafe { mem::transmute(buffer) };
    Ok(value)
}

pub fn read_i16(reader: &mut io::Read) -> io::Result<i16> {
    let mut buffer = [0 as u8; 2];
    try!(reader.read_exact(&mut buffer));

    let value: i16 = unsafe { mem::transmute(buffer) };
    Ok(value)
}

pub fn read_i32(reader: &mut io::Read) -> io::Result<i32> {
    let mut buffer = [0 as u8; 4];
    try!(reader.read_exact(&mut buffer));

    let value: i32 = unsafe { mem::transmute(buffer) };
    Ok(value)
}

pub fn read_i64(reader: &mut io::Read) -> io::Result<i64> {
    let mut buffer = [0 as u8; 8];
    try!(reader.read_exact(&mut buffer));

    let value: i64 = unsafe { mem::transmute(buffer) };
    Ok(value)
}

pub fn read_f32(reader: &mut io::Read) -> io::Result<f32> {
    let mut buffer = [0 as u8; 4];
    try!(reader.read_exact(&mut buffer));

    let value: f32 = unsafe { mem::transmute(buffer) };
    Ok(value)
}

pub fn read_f64(reader: &mut io::Read) -> io::Result<f64> {
    let mut buffer = [0 as u8; 8];
    try!(reader.read_exact(&mut buffer));

    let value: f64 = unsafe { mem::transmute(buffer) };
    Ok(value)
}

macro_rules! read_array {
    ($reader:expr, $len:expr, $t:ty, $buffer:expr) => {
        {
        let mut chunks: Vec<$t> = Vec::new();
        let mut len = $len;

        while len > 0 {
            let mut buffer = $buffer;
            try!($reader.read_exact(&mut buffer));

            chunks.push(unsafe { mem::transmute(buffer) });

            len = len - buffer.len();
        }

        Ok(chunks)
        }
    };
}

pub fn read_u8_array(reader: &mut io::Read) -> io::Result<Vec<u8>> {
    let (len, encoded, compressed_length) = try!(read_array_header(reader));
    if encoded {
        unimplemented!()
    } else {
        read_array!(reader, len, u8, [0 as u8; 1])
    }
}

pub fn read_u32_array(reader: &mut io::Read) -> io::Result<Vec<u32>> {
    let (len, encoded, compressed_length) = try!(read_array_header(reader));
    if encoded {
        unimplemented!()
    } else {
        read_array!(reader, len, u32, [0 as u8; 4])
    }
}

pub fn read_i16_array(reader: &mut io::Read) -> io::Result<Vec<i16>> {
    let (len, encoded, compressed_length) = try!(read_array_header(reader));
    if encoded {
        unimplemented!()
    } else {
        read_array!(reader, len, i16, [0 as u8; 2])
    }
}

pub fn read_i32_array(reader: &mut io::Read) -> io::Result<Vec<i32>> {
    let (len, encoded, compressed_length) = try!(read_array_header(reader));
    if encoded {
        unimplemented!()
    } else {
        read_array!(reader, len, i32, [0 as u8; 4])
    }
}

pub fn read_i64_array(reader: &mut io::Read) -> io::Result<Vec<i64>> {
    let (len, encoded, compressed_length) = try!(read_array_header(reader));
    if encoded {
        unimplemented!()
    } else {
        read_array!(reader, len, i64, [0 as u8; 8])
    }
}

pub fn read_f32_array(reader: &mut io::Read) -> io::Result<Vec<f32>> {
    let (len, encoded, compressed_length) = try!(read_array_header(reader));
    if encoded {
        unimplemented!()
    } else {
        read_array!(reader, len, f32, [0 as u8; 4])
    }
}

pub fn read_f64_array(reader: &mut io::Read) -> io::Result<Vec<f64>> {
    let (len, encoded, compressed_length) = try!(read_array_header(reader));
    if encoded {
        unimplemented!()
    } else {
        read_array!(reader, len, f64, [0 as u8; 8])
    }
}

pub fn read_array_header(reader: &mut io::Read) -> io::Result<(usize, bool, usize)> {
    Ok(
        (try!(read_u32(reader)) as usize, // Length
         try!(read_u32(reader)) != 0,     // Encoding bool
         try!(read_u32(reader)) as usize  // compressed length
        )
    )
}

fn _read_string_len(reader: &mut io::Read, len: usize) -> io::Result<String> {
    let bytes: io::Result<Vec<u8>> = read_array!(reader, len, u8, [0 as u8; 1]);
    let string = unsafe { String::from_utf8_unchecked(try!(bytes)) };
    Ok(string)
}

pub fn read_ubyte_string(reader: &mut io::Read) -> io::Result<String> {
    let len = try!(read_u8(reader)) as usize;
    _read_string_len(reader, len)
}


pub fn read_string(reader: &mut io::Read) -> io::Result<String> {
    let len = try!(read_u32(reader)) as usize;
    _read_string_len(reader, len)
}

#[cfg(test)]
mod tests {
    use std::io;
    use super::*;

    #[test]
    fn test_read_string() {
        // The length of the string is indicated by a byte value before it
        let string = "\x08\x00\x00\x00A string".as_bytes();

        let mut reader = io::Cursor::new(string);
        assert_eq!(read_string(&mut reader).unwrap(), "A string");
    }
}
