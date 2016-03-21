use std::io;
use std::io::ErrorKind;
use std::mem;

pub struct FbxLoader {
   current_offset: usize
}

fn _verify_fbx_header(reader: &mut io::Read) -> io::Result<usize> {
    let expected_header = "Kaydara FBX Binary\x20\x20\x00\x1a\x00";
    let mut header_buffer = [0 as u8; 23];
    try!(reader.read_exact(&mut header_buffer));

    if header_buffer == expected_header.as_bytes() {
        Ok(23)
    } else {
        Err(io::Error::new(ErrorKind::InvalidData, "Invalid FBX header"))
    }
}

fn _read_u32(reader: &mut io::Read) -> io::Result<u32> {
    let mut version_buffer = [0 as u8; 4];
    try!(reader.read_exact(&mut version_buffer));

    let value: u32 = unsafe { mem::transmute(version_buffer) };
    Ok(value)
}

fn _read_u8(reader: &mut io::Read) -> io::Result<u8> {
    let mut version_buffer = [0 as u8; 1];
    try!(reader.read_exact(&mut version_buffer));

    Ok(version_buffer[0])
}

fn _read_string(reader: &mut io::Read) -> io::Result<String> {
    let mut string = String::new();

    let mut len = try!(_read_u8(reader));

    while len > 0 {
        len -= 1;
        let mut buffer = [0 as u8; 1];
        try!(reader.read_exact(&mut buffer));
        string.push(buffer[0] as char)
    }

    Ok(string)
}

impl FbxLoader {
    pub fn new() -> Self {
        FbxLoader { current_offset: 0 }
    }

    fn verify_fbx_header(&mut self, reader: &mut io::Read) -> io::Result<()> {
        self.current_offset += try!(_verify_fbx_header(reader));
        Ok(())
    }

    fn read_fbx_version(&mut self, reader: &mut io::Read) -> io::Result<u32> {
        let version: u32 = try!(_read_u32(reader));

        self.current_offset += 4;
        Ok(version)
    }

    fn read_element(&mut self, reader: &mut io::Read) -> io::Result<Option<usize>> {
        // [0] = offset at which this block ends - u32
        let end_offset = try!(_read_u32(reader));

        if end_offset == 0 {
            return Ok(None)
        }

        // [1] = number of props in the scope -  u32
        // [2] = length of the property list - u32
        let property_count  = try!(_read_u32(reader));
        let property_length = try!(_read_u32(reader));



        Ok(None)
    }

    pub fn parse(&mut self, reader: &mut io::Read) -> io::Result<()> {
        try!(self.verify_fbx_header(reader));
        let fbx_version = try!(self.read_fbx_version(reader));

        Ok(()) }
}

#[cfg(test)]
mod tests {
    use std::io;
    use super::{ _read_string, _verify_fbx_header };
    use super::*;

    #[test]
    fn test_read_string() {
        // The length of the string is indicated by a byte value before it
        let string = "\x08A string".as_bytes();

        let mut reader = io::Cursor::new(string);
        assert_eq!(_read_string(&mut reader).unwrap(), "A string");
    }

    #[test]
    fn test_header_ok() {
        let correct_fbx_header = vec![0x4b, 0x61, 0x79, 0x64,
                                   0x61, 0x72, 0x61, 0x20,
                                   0x46, 0x42, 0x58, 0x20,
                                   0x42, 0x69, 0x6e, 0x61,
                                   0x72, 0x79, 0x20, 0x20,
                                   0x00, 0x1a, 0x00 ];

        let mut reader = io::Cursor::new(correct_fbx_header);
        let result = _verify_fbx_header(&mut reader);

        assert_eq!(result.unwrap(), 23);
    }

    #[test]
    fn test_header_fail() {
        let incorrect_fbx_header = vec![0xab, 0x61, 0x79, 0x64,
                                   0x61, 0x92, 0x61, 0x20,
                                   0x46, 0x42, 0x58, 0x20,
                                   0x42, 0x19, 0x6e, 0x61,
                                   0x72, 0x79, 0x20, 0x20,
                                   0x00, 0x1a, 0x00 ];

        let mut reader = io::Cursor::new(incorrect_fbx_header);

        assert!(_verify_fbx_header(&mut reader).is_err());
    }

    #[test]
    fn test_read_version() {
        let version = vec![0x84, 0x1c, 0x00, 0x00];

        let mut reader = io::Cursor::new(version);

        assert_eq!(FbxLoader::new().read_fbx_version(&mut reader).unwrap(), 7300);

    }

    #[test]
    fn test_parse_rejects_invalid_fbx() {
        let incorrect_fbx = vec![0xab, 0x61, 0x79, 0x64,
                                   0x61, 0x92, 0x61, 0x20,
                                   0x46, 0x42, 0x58, 0x20,
                                   0x42, 0x19, 0x6e, 0x61,
                                   0x72, 0x79, 0x20, 0x20];

        let mut reader = io::Cursor::new(incorrect_fbx);

        assert!(FbxLoader::new().parse(&mut reader).is_err());
    }
}
