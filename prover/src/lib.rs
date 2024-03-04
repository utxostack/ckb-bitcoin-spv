//! Bitcoin simplified payment verification (the prover part).

mod block;
mod dummy_service;
mod result;
pub(crate) mod utilities;

#[cfg(test)]
mod tests;

pub use block::BlockProofGenerator;
pub use dummy_service::DummyService;
pub use result::{Error, Result};
