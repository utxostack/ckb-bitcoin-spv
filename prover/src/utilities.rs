//! Utilities for internal usage.

use std::{fs::File, io::Read as _, path::Path};

use bitcoin::consensus::{deserialize, Decodable};

use crate::{Error, Result};

/// Decode a struct from a binary file.
pub fn decode_from_bin_file<T: Decodable, P: AsRef<Path>>(bin_file: P) -> Result<T> {
    File::open(bin_file.as_ref())
        .and_then(|mut file| {
            let mut data = Vec::new();
            file.read_to_end(&mut data).map(|_| data)
        })
        .map_err(|err| {
            let msg = format!(
                "faild to load binary file \"{}\" since {err}",
                bin_file.as_ref().display()
            );
            Error::other(msg)
        })
        .and_then(|data| decode_from_slice(&data))
}

/// Decode a struct from a slice.
pub fn decode_from_slice<T: Decodable>(slice: &[u8]) -> Result<T> {
    deserialize(slice).map_err(|err| {
        let msg = format!("faild to parse bytes of a block since {err}");
        Error::other(msg)
    })
}
