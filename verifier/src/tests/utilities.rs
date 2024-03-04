use std::{fs::File, io::Read as _, path::Path, vec::Vec};

use bitcoin::consensus::{deserialize, Decodable};

pub(crate) fn decode_from_slice<T: Decodable>(slice: &[u8]) -> T {
    deserialize(slice).unwrap()
}

pub(crate) fn decode_from_bin_file<T: Decodable, P: AsRef<Path>>(bin_file: P) -> T {
    let v = File::open(bin_file.as_ref())
        .and_then(|mut file| {
            let mut data = Vec::new();
            file.read_to_end(&mut data).map(|_| data)
        })
        .unwrap();
    decode_from_slice(&v)
}
