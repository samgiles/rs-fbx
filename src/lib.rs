extern crate flate2;
mod reader;

use reader::*;

use std::io;
use std::io::{ ErrorKind, SeekFrom };

// at the end of each nested block, there is a NUL record to indicate
// that the sub-scope exists (i.e. to distinguish between P: and P : {})
// this NULL record is 13 bytes long.
const BLOCK_SENTINEL_LENGTH: usize = 13;

#[derive(Debug)]
pub struct FbxElement {
    id: String,
    properties: Vec<PropertyType>,
    elements: Vec<FbxElement>,
}

impl FbxElement {
    fn new(id: String, properties: Vec<PropertyType>, elements: Vec<FbxElement>) -> Self {
        FbxElement {
            id: id,
            properties: properties,
            elements: elements,
        }
    }
}

pub struct FbxLoader;

fn _verify_fbx_header(reader: &mut io::Read) -> io::Result<()> {
    let expected_header = "Kaydara FBX Binary\x20\x20\x00\x1a\x00";
    let mut header_buffer = [0 as u8; 23];
    try!(reader.read_exact(&mut header_buffer));

    if header_buffer == expected_header.as_bytes() {
        Ok(())
    } else {
        Err(io::Error::new(ErrorKind::InvalidData, "Invalid FBX header"))
    }
}

#[derive(Debug)]
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
        FbxLoader
    }

    fn verify_fbx_header(&self, reader: &mut io::Read) -> io::Result<()> {
        _verify_fbx_header(reader)
    }

    fn read_fbx_version(&self, reader: &mut io::Read) -> io::Result<u32> {
        let version: u32 = try!(read_u32(reader));
        Ok(version)
    }

    fn read_element<T: io::Read + io::Seek>(&self, reader: &mut T) -> io::Result<Option<FbxElement>> {
        // [0] = offset at which this block ends - u32
        let end_offset = try!(read_u32(reader)) as u64;

        if end_offset == 0 {
            return Ok(None);
        }

        // [1] = number of props in the scope -  u32
        // [2] = length of the property list - u32
        let property_count  = try!(read_u32(reader)) as usize;
        let _ = try!(read_u32(reader));

        let element_id = try!(read_ubyte_string(reader));
        let mut element_properties: Vec<PropertyType> = Vec::with_capacity(property_count);

        for _ in 0..property_count {
            let data_type = try!(read_u8(reader));

            // assumes little endian
            let property = match data_type {
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
                    PropertyType::BinaryData(try!(read_binary_data(reader)))
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
                    panic!("unknown property type 0x{:02x} - invalid FBX file", data_type)
                }
            };


            element_properties.push(property);
        }


        let mut child_elements = Vec::new();

        if try!(self.position(reader)) < end_offset {
            while try!(self.position(reader)) < end_offset - (BLOCK_SENTINEL_LENGTH as u64) {
                match try!(self.read_element(reader)) {
                    Some(element) => {
                        child_elements.push(element);
                    },
                    _ => { }
                }
            }

            let mut empty_block = [0 as u8; BLOCK_SENTINEL_LENGTH];
            try!(reader.read_exact(&mut empty_block));
            assert!(empty_block == [0 as u8; BLOCK_SENTINEL_LENGTH]);
        }

        if try!(self.position(reader)) != end_offset {
            return Err(
                io::Error::new(ErrorKind::InvalidData,
                               "Did not reach the end of the scope, corrupt FBX?")
                );
        }

        Ok(Some(FbxElement::new(element_id, element_properties, child_elements)))
    }

    fn position<T: io::Read + io::Seek>(&self, reader: &mut T) -> io::Result<u64> {
        reader.seek(SeekFrom::Current(0))
    }

    pub fn parse<T: io::Read + io::Seek>(&self, reader: &mut T) -> io::Result<FbxElement> {
        try!(self.verify_fbx_header(reader));
        let _ = try!(self.read_fbx_version(reader));

        let mut root_elements = Vec::new();

        loop {
            let element = try!(self.read_element(reader));

            match element {
                Some(element) => {
                    root_elements.push(element);
                },
                None => {
                    break;
                }
            }
        }

        Ok(FbxElement::new("".to_owned(), Vec::new(), root_elements))
    }
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
        _verify_fbx_header(&mut reader).unwrap();
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
