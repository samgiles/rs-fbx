use std::io;
use std::io::ErrorKind;
use std::mem;

pub fn verify_fbx_header(reader: &mut io::Read) -> io::Result<()> {
    let expected_header = "Kaydara FBX Binary\x20\x20\x00\x1a\x00";
    let mut header_buffer = [0 as u8; 23];
    try!(reader.read_exact(&mut header_buffer));

    if header_buffer == expected_header.as_bytes() {
        Ok(())
    } else {
        Err(io::Error::new(ErrorKind::InvalidData, "Invalid FBX header"))
    }
}

pub fn read_fbx_version(reader: &mut io::Read) -> io::Result<u32> {
    let mut version_buffer = [0 as u8; 4];
    try!(reader.read_exact(&mut version_buffer));

    let version: u32 = unsafe { mem::transmute(version_buffer) };
    Ok(version)
}

pub fn parse(reader: &mut io::Read) -> io::Result<()> {
    try!(verify_fbx_header(reader));

    let fbx_version = try!(read_fbx_version(reader));

    Ok(())
}


#[cfg(test)]
mod tests {
    use std::io;
    use super::*;

    #[test]
    fn test_verify_fbx_header_correct() {
        let correct_fbx_header = vec![0x4b, 0x61, 0x79, 0x64,
                                   0x61, 0x72, 0x61, 0x20,
                                   0x46, 0x42, 0x58, 0x20,
                                   0x42, 0x69, 0x6e, 0x61,
                                   0x72, 0x79, 0x20, 0x20,
                                   0x00, 0x1a, 0x00 ];

        let mut reader = io::Cursor::new(correct_fbx_header);

        assert!(verify_fbx_header(&mut reader).is_ok());
    }

    #[test]
    fn test_verify_fbx_header_incorrect() {
        let incorrect_fbx_header = vec![0xab, 0x61, 0x79, 0x64,
                                   0x61, 0x92, 0x61, 0x20,
                                   0x46, 0x42, 0x58, 0x20,
                                   0x42, 0x19, 0x6e, 0x61,
                                   0x72, 0x79, 0x20, 0x20,
                                   0x00, 0x1a, 0x00 ];

        let mut reader = io::Cursor::new(incorrect_fbx_header);

        assert!(verify_fbx_header(&mut reader).is_err());
    }

    #[test]
    fn test_read_version() {
        let version = vec![0x84, 0x1c, 0x00, 0x00];

        let mut reader = io::Cursor::new(version);

        assert_eq!(read_fbx_version(&mut reader).unwrap(), 7300);

    }

    #[test]
    fn test_parse_rejects_invalid_fbx() {
        let incorrect_fbx = vec![0xab, 0x61, 0x79, 0x64,
                                   0x61, 0x92, 0x61, 0x20,
                                   0x46, 0x42, 0x58, 0x20,
                                   0x42, 0x19, 0x6e, 0x61,
                                   0x72, 0x79, 0x20, 0x20];

        let mut reader = io::Cursor::new(incorrect_fbx);

        assert!(parse(&mut reader).is_err());
    }
}
