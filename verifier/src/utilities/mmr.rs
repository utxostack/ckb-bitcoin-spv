//! The utilities for [Merkle Mountain Ranges (MMR)].
//!
//! [Merkle Mountain Ranges (MMR)]: https://github.com/opentimestamps/opentimestamps-server/blob/master/doc/merkle-mountain-range.md

use alloc::format;

use bitcoin::pow::Target;
use ckb_mmr::{Error as MMRError, Merge, MerkleProof, Result as MMRResult, MMR};
use ethereum_types::U256;

use crate::{
    core::cmp::PartialEq,
    types::{core, packed, prelude::*},
};

pub use ckb_mmr as lib;

/// A struct to implement MMR `Merge` trait.
pub struct MergeHeaderDigest;
/// MMR root.
pub type ClientRootMMR<S> = MMR<packed::HeaderDigest, MergeHeaderDigest, S>;
/// MMR proof.
pub type MMRProof = MerkleProof<packed::HeaderDigest, MergeHeaderDigest>;

/// Merges two hashes.
pub fn hash_concat(lhs: &core::Hash, rhs: &core::Hash) -> core::Hash {
    let mut data = [0u8; 64];
    data[..32].copy_from_slice(lhs.as_ref());
    data[32..].copy_from_slice(rhs.as_ref());
    core::Hash::hash(&data)
}

impl core::HeaderDigest {
    /// Creates a new header digest for a leaf node.
    pub fn new_leaf(height: u32, header: &core::Header) -> Self {
        let block_hash = header.block_hash().into();
        let target: Target = header.bits.into();
        let blockwork = U256::from_little_endian(&target.to_work().to_le_bytes());
        Self {
            min_height: height,
            max_height: height,
            partial_chain_work: blockwork,
            children_hash: block_hash,
        }
    }
}

impl<'r> packed::HeaderDigestReader<'r> {
    /// Calculates the MMR hash root for the current MMR node.
    pub fn calc_mmr_hash(&self) -> core::Hash {
        core::Hash::hash(self.as_slice())
    }
}

impl packed::HeaderDigest {
    /// Calculates the MMR hash root for the current MMR node.
    pub fn calc_mmr_hash(&self) -> core::Hash {
        self.as_reader().calc_mmr_hash()
    }
}

impl PartialEq for packed::HeaderDigest {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Merge for MergeHeaderDigest {
    type Item = packed::HeaderDigest;

    fn merge(lhs: &Self::Item, rhs: &Self::Item) -> MMRResult<Self::Item> {
        // 1. Check block heights.
        let lhs_end: u32 = lhs.max_height().unpack();
        let rhs_start: u32 = rhs.min_height().unpack();
        if lhs_end + 1 != rhs_start {
            let errmsg = format!(
                "failed since the headers isn't continuous ([-,{lhs_end}], [{rhs_start},-])"
            );
            return Err(MMRError::MergeError(errmsg));
        }
        let lhs_work = lhs.partial_chain_work().unpack();
        let rhs_work = rhs.partial_chain_work().unpack();
        let partial_chain_work = lhs_work + rhs_work;
        let children_hash = hash_concat(&lhs.calc_mmr_hash(), &rhs.calc_mmr_hash());
        Ok(Self::Item::new_builder()
            .min_height(lhs.min_height())
            .max_height(rhs.max_height())
            .partial_chain_work(partial_chain_work.pack())
            .children_hash(children_hash.pack())
            .build())
    }

    fn merge_peaks(lhs: &Self::Item, rhs: &Self::Item) -> MMRResult<Self::Item> {
        Self::merge(rhs, lhs)
    }
}
