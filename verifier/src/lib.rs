//! Bitcoin simplified payment verification (the verifier part).

#![no_std]

#[cfg(not(any(feature = "std", feature = "no-std")))]
compile_error!("at least one of the `std` or `no-std` features must be enabled");

extern crate alloc;
extern crate core;

#[macro_use]
mod log;

pub mod error;
pub mod types;
pub mod utilities;

pub extern crate molecule;

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests;
