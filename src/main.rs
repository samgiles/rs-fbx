extern crate fbx;

use std::io;
use std::fs;
use fbx::*;

fn main() {
    let mut file = fs::File::open("370zfbx.FBX").unwrap();

    let loader = FbxLoader::new();

    let fbx = loader.parse(&mut file);

    println!("{:?}", fbx);
}
