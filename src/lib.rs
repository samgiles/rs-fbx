mod reader;

use reader::*;

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

pub enum PropertyType {
    I16(i16),
    Bool(bool),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    BinaryData(Vec<u8>),
    StringData(String),
    ArrayF32(Vec<f32>),
    ArrayI32(Vec<i32>),
    ArrayF64(Vec<f64>),
    ArrayI64(Vec<i64>),
    ArrayBool(Vec<u8>),
    ArrayU8(Vec<u8>)
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
        let version: u32 = try!(read_u32(reader));

        self.current_offset += 4;
        Ok(version)
    }

    fn read_element(&mut self, reader: &mut io::Read) -> io::Result<Option<usize>> {
        // [0] = offset at which this block ends - u32
        let end_offset = try!(read_u32(reader));

        if end_offset == 0 {
            return Ok(None)
        }

        // [1] = number of props in the scope -  u32
        // [2] = length of the property list - u32
        let property_count  = try!(read_u32(reader)) as usize;
        let property_length = try!(read_u32(reader)) as usize;

        let element_id = try!(read_string(reader));
        let element_property_types: Vec<u8> = Vec::with_capacity(property_count);
        let element_property_data:  Vec<Option<()>> = Vec::with_capacity(property_count);

        for index in 0..property_count {
            let data_type = try!(read_u8(reader));

            // assumes little endian
            let _ = match data_type {
                0x59 /* i16 */ => {
                    PropertyType::I16(try!(read_i16(reader)))
                },
                0x43 /* 1 byte bool */ => {
                    PropertyType::Bool(try!(read_u8(reader)) != 0)
                },
                0x49 /* i32 */ => {
                    PropertyType::I32(try!(read_i32(reader)))
                },
                0x46 /* f32 */ => {
                    PropertyType::F32(try!(read_f32(reader)))
                },
                0x44 /* f64 */ => {
                    PropertyType::F64(try!(read_f64(reader)))
                },
                0x4c /* i64 */ => {
                    PropertyType::I64(try!(read_i64(reader)))
                },
                0x52 /* binary data */ => {
                    PropertyType::BinaryData(try!(read_u8_array(reader)))
                },
                0x53 /* string data */ => {
                    PropertyType::StringData(try!(read_string(reader)))
                },
                0x66 /* array(f32) */ => {
                    PropertyType::ArrayF32(try!(read_f32_array(reader)))
                },
                0x69 /* array(i32) */ => {
                    PropertyType::ArrayI32(try!(read_i32_array(reader)))
                },
                0x64 /* array(f64) */ => {
                    PropertyType::ArrayF64(try!(read_f64_array(reader)))
                },
                0x6c /* array(i64) */ => {
                    PropertyType::ArrayI64(try!(read_i64_array(reader)))
                },
                0x62 /* array(bool) */ => {
                    PropertyType::ArrayBool(try!(read_u8_array(reader)))
                },
                0x63 /* array(u8) */ => {
                    PropertyType::ArrayU8(try!(read_u8_array(reader)))
                },
                _ => {
                    panic!("unknown property type - invalid FBX file")
                }
            };
        }



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
    use reader::read_string;
    use super::*;
    use super::_verify_fbx_header;


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
