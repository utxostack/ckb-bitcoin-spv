//! Generate block-related proofs.

use std::{collections::HashSet, path::Path};

use bitcoin::{consensus::serialize, Block, MerkleBlock, Transaction};

use crate::{utilities, Error, Result};

#[derive(Clone)]
pub struct BlockProofGenerator {
    original: Block,
}

impl From<Block> for BlockProofGenerator {
    fn from(block: Block) -> Self {
        Self::new(block)
    }
}

impl AsRef<Block> for BlockProofGenerator {
    fn as_ref(&self) -> &Block {
        &self.original
    }
}

/// A generator, which are used to generate block-related proofs.
impl BlockProofGenerator {
    /// Create a new block proof generator.
    pub fn new(block: Block) -> Self {
        Self { original: block }
    }

    /// Load a block from its binary data.
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        utilities::decode_from_slice(slice).map(Self::new)
    }

    /// Load a block from a file which contains its binary data.
    pub fn from_bin_file<P: AsRef<Path>>(bin_file: P) -> Result<Self> {
        utilities::decode_from_bin_file(bin_file).map(Self::new)
    }

    /// Get transaction.
    pub fn get_transaction(&self, index: usize) -> Result<&Transaction> {
        let block = self.as_ref();
        block.txdata.get(index).ok_or_else(|| {
            let msg = format!(
                "block {:#x} doesn't have {index}-th transaction",
                block.block_hash()
            );
            Error::other(msg)
        })
    }

    /// Generate transaction outputs proof.
    pub fn generate_txoutproof_via_indexes(&self, indexes: &[u32]) -> Result<Vec<u8>> {
        let block = self.as_ref();
        let match_txids = indexes
            .iter()
            .map(|i| *i as usize)
            .map(|i| {
                block
                    .txdata
                    .get(i)
                    .ok_or_else(|| {
                        let msg = format!(
                            "block {:#x} doesn't have {i}-th transaction",
                            block.block_hash()
                        );
                        Error::other(msg)
                    })
                    .map(|tx| tx.txid())
            })
            .collect::<Result<HashSet<_>>>()?;
        let mb = MerkleBlock::from_block_with_predicate(block, |t| match_txids.contains(t));
        Ok(serialize(&mb))
    }
}
