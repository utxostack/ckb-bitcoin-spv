//! The essential rust types.
//!
//! [Packed bytes] are not enough for all usage scenarios.
//!
//! This module provides essential rust types.
//!
//! Most of them is composed of [those packed bytes] or can convert between `self` and [those packed bytes].
//!
//! [Packed bytes]: ../packed/index.html
//! [those packed bytes]: ../packed/index.html

#[cfg(feature = "std")]
use alloc::fmt;
use alloc::vec::Vec;

pub use bitcoin::{
    blockdata::{block::Header, transaction::Transaction},
    hash_types::Txid,
    merkle_tree::MerkleBlock,
    pow::{CompactTarget, Target},
};
pub use bitcoin_hashes::sha256d::Hash;
pub use ethereum_types::U256;
pub use molecule::bytes::Bytes;

use crate::types::packed;

//
// Proofs
//

/// Merkle Node of Merkle Mountain Ranges.
///
/// Ref: [`MmrProof`]
#[derive(Clone)]
pub struct HeaderDigest {
    /// The min height of the headers in MMR.
    pub min_height: u32,
    /// The max height of the headers in MMR.
    pub max_height: u32,
    /// Chain work between min height and max height.
    pub partial_chain_work: U256,
    /// The block hash for leaves; otherwise, the hash of children nodes.
    pub children_hash: Hash,
}

/// Merkle Mountain Ranges (MMR) Proof.
///
/// See [Merkle Mountain Ranges] for more details.
///
/// [Merkle Mountain Ranges]: https://github.com/opentimestamps/opentimestamps-server/blob/master/doc/merkle-mountain-range.md
pub type MmrProof = Vec<HeaderDigest>;

//
// Cells Data
//

/// The SPV info cell.
#[derive(Clone)]
pub struct SpvInfo {
    /// The ID of the latest SPV client cell.
    pub tip_client_id: u8,
}

/// The SPV client cell.
#[derive(Clone)]
pub struct SpvClient {
    /// An unique ID of the SPV client cell.
    pub id: u8,
    /// The root of the latest header.
    pub tip_block_hash: Hash,
    /// The MMR root of headers between height `min_height` and height `max_height`.
    pub headers_mmr_root: HeaderDigest,
    /// The target adjusts on every 2016th block, SpvClient stores the latest one.
    pub target_adjust_info: packed::TargetAdjustInfo,
}

//
// Script Args
//

/// The args for the type script of the SPV info cell and SPV client cells.
#[derive(Clone)]
pub struct SpvTypeArgs {
    pub type_id: Hash,
    /// How many SPV client cells that use current type script.
    ///
    /// N.B. Exclude the SPV info cell.
    pub clients_count: u8,
    /// Bit flags to control features.
    ///
    /// From high to low:
    /// - Set 0-th bit to true, to disable difficulty checks.
    /// - Other bits are reserved.
    pub flags: u8,
}

#[cfg(feature = "std")]
impl fmt::Display for HeaderDigest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ headers-range: [{}, {}], hash: {:#x} }}",
            self.min_height, self.max_height, self.children_hash
        )
    }
}

#[cfg(feature = "std")]
impl fmt::Display for SpvClient {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, tip: {:#x}, mmr-root: {} }}",
            self.id, self.tip_block_hash, self.headers_mmr_root
        )
    }
}
