//! Errors.

use thiserror::Error;

use ckb_bitcoin_spv_verifier::utilities::mmr;

#[derive(Debug, Error)]
pub enum Error {
    #[error("mmr error: {0}")]
    Mmr(#[from] mmr::lib::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn other<S: ToString>(arg: S) -> Self {
        Self::Other(arg.to_string())
    }
}
