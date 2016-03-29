# Rust FBX file reader

This is a simple implementation of an FBX file loader in Rust.

I've only tested it on a couple of files so far, so have no idea how robust
this is.  It's a WIP.

*Warning: This is a work in progress!*


# Usage

```RUST
use fbx::FbxLoader;
use std::io;
use std::fs;

let file = fs::File::open("your.fbx").unwrap();
let mut buffered_reader = io::BufReader::new(file);
let loader = FbxLoader::new();

let fbx = loader.parse(&mut buffered_reader);
```

# License

GPL v3
