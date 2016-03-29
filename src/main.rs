extern crate fbx;

use std::io;
use std::fs;
use fbx::*;

fn main() {
    let mut file = fs::File::open("370zfbx.FBX").unwrap();

    let mut buffered_reader = io::BufReader::new(file);

    let loader = FbxLoader::new();

    let fbx = loader.parse(&mut buffered_reader);

    println!("{:?}", fbx.unwrap());
}
