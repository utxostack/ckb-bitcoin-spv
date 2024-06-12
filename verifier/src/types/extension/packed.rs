//! Extensions for packed types.

use alloc::{vec, vec::Vec};

use bitcoin::{
    blockdata::constants::DIFFCHANGE_INTERVAL,
    consensus::{deserialize, encode::Error as EncodeError, serialize},
};
use molecule::bytes::Bytes;

use crate::{
    core::result::Result,
    error::{BootstrapError, UpdateError, VerifyTxError},
    types::{core, packed, prelude::*},
    utilities::{
        bitcoin::calculate_next_target,
        mmr::{
            self,
            lib::{leaf_index_to_mmr_size, leaf_index_to_pos},
        },
    },
};

impl packed::TargetAdjustInfoReader<'_> {
    /// Decodes a packed type to a rust type.
    pub fn decode(&self) -> Result<(u32, core::CompactTarget), EncodeError> {
        let start_time: u32 = deserialize(&self.as_slice()[..4])?;
        let next_bits: core::CompactTarget = deserialize(&self.as_slice()[4..])?;
        Ok((start_time, next_bits))
    }
}

impl packed::TargetAdjustInfo {
    /// Decodes a packed type to a rust type.
    pub fn decode(&self) -> Result<(u32, core::CompactTarget), EncodeError> {
        self.as_reader().decode()
    }

    /// Encodes a rust type to a packed type.
    pub fn encode(start_time: u32, next_bits: core::CompactTarget) -> Self {
        let start_time_bytes = serialize(&start_time);
        let next_bits_bytes = serialize(&next_bits);
        let mut array = [0u8; 8];
        array[..4].copy_from_slice(&start_time_bytes);
        array[4..].copy_from_slice(&next_bits_bytes);
        Self::new_unchecked(Bytes::from(array.to_vec()))
    }
}

impl packed::SpvBootstrap {
    /// Initializes a new SPV client.
    ///
    /// The height of the input header should be multiples of [`DIFFCHANGE_INTERVAL`].
    ///
    /// The client ID, which constructs from this method, is always be `0`.
    ///
    /// Ref:
    /// - [How often does the network difficulty change?](https://en.bitcoin.it/wiki/Difficulty#How_often_does_the_network_difficulty_change.3F)
    ///
    /// [`DIFFCHANGE_INTERVAL`]: https://docs.rs/bitcoin/latest/bitcoin/blockdata/constants/constant.DIFFCHANGE_INTERVAL.html
    pub fn initialize_spv_client(&self) -> Result<core::SpvClient, BootstrapError> {
        let height: u32 = self.height().unpack();
        if height % DIFFCHANGE_INTERVAL != 0 {
            error!("the started height {height} should be multiples of {DIFFCHANGE_INTERVAL}");
            return Err(BootstrapError::Height);
        }
        let header: core::Header =
            deserialize(&self.header().raw_data()).map_err(|_| BootstrapError::DecodeHeader)?;
        // Verify POW: just trust the input header.
        // TODO Check constants::FLAG_DISABLE_DIFFICULTY_CHECK before return errors.
        let block_hash = header
            .validate_pow(header.target())
            .map_err(|_| BootstrapError::Pow)?
            .into();
        let target_adjust_info = packed::TargetAdjustInfo::encode(header.time, header.bits);
        let digest = core::HeaderDigest::new_leaf(height, &header);
        let client = core::SpvClient {
            id: 0,
            tip_block_hash: block_hash,
            headers_mmr_root: digest,
            target_adjust_info,
        };
        Ok(client)
    }
}

impl packed::SpvClient {
    /// Verifies a new client.
    ///
    /// Checks:
    /// - Check headers:
    ///     - Check previous block hashes.
    ///     - Check the target adjust info.
    ///     - Check POW.
    /// - Check MMR root:
    ///     - All appeneded headers are included in the new MMR root.
    ///     - All headers, which are included in the old MMR root,
    ///       also are included in the new MMR root.
    ///     - No more headers are appended into the new MMR root.
    /// - Check new client:
    ///     - ID should be the same.
    ///     - Check the new tip block hash.
    ///     - The min height should be the same.
    ///     - Check the new max height.
    ///     - Check the target adjust info.
    pub fn verify_new_client(
        &self,
        packed_new_client: &Self,
        update: packed::SpvUpdate,
        flags: u8,
    ) -> Result<(), UpdateError> {
        let old_client = self.unpack();
        let new_client = packed_new_client.unpack();
        info!("old client is {old_client}");
        info!("new client is {new_client}");

        // Check Headers
        let headers = update.headers();
        if headers.is_empty() {
            error!("failed: update has no headers");
            return Err(UpdateError::EmptyHeaders);
        }
        debug!("update has {} headers", headers.len());
        let mut digests = Vec::with_capacity(headers.len());
        let mut new_tip_block_hash = old_client.tip_block_hash;
        let mut new_max_height = old_client.headers_mmr_root.max_height;
        let mut new_info = old_client
            .target_adjust_info
            .decode()
            .map_err(|_| UpdateError::DecodeTargetAdjustInfo)?;
        trace!("tip block hash: {new_tip_block_hash:#x}, max height: {new_max_height}");
        for header in update.headers().as_reader().iter() {
            new_max_height += 1;
            let header: core::Header =
                deserialize(header.raw_data()).map_err(|_| UpdateError::DecodeHeader)?;
            let block_hash = header.prev_blockhash.into();
            if new_tip_block_hash != block_hash {
                error!("failed: headers are uncontinuous");
                return Err(UpdateError::UncontinuousHeaders);
            }
            // Check the target adjust info.
            if new_info.1 != header.bits {
                log_if_enabled!(|Error| {
                    let expected = new_info.1.to_consensus();
                    let actual = header.bits.to_consensus();
                    error!(
                        "failed: invalid difficulty for header-{new_max_height}, \
                        expect {expected} but got {actual}"
                    );
                });

                // For mainnet and signet, `header.bits` should be as the same as `new_info.1`.
                // But for testnet, it could be not.
                if core::BitcoinChainType::Testnet != flags.into() {
                    return Err(UpdateError::Difficulty);
                }
            }
            // Check POW.
            new_tip_block_hash = header
                .validate_pow(header.bits.into())
                .map_err(|_| UpdateError::Pow)?
                .into();

            // Update the target adjust info.
            {
                match (new_max_height + 1) % DIFFCHANGE_INTERVAL {
                    // Next block is the first block for a new difficulty.
                    0 => {
                        // See the above check:
                        // - For mainnet, `header.bits` should be as the same as `new_info.1`.
                        // - But for testnet, it could be not.
                        let prev_target = header.bits.into();
                        let next_target =
                            calculate_next_target(prev_target, new_info.0, header.time, flags);
                        new_info.1 = next_target.to_compact_lossy();
                    }
                    // Current block is the first block for a new difficulty.
                    1 => {
                        new_info.0 = header.time;
                    }
                    _ => {}
                }
            }
            let digest = core::HeaderDigest::new_leaf(new_max_height, &header);
            trace!(
                "tip block hash: {new_tip_block_hash:#x}, max height: {new_max_height}, \
                digest: {digest}"
            );
            digests.push(digest.pack());
        }

        // Check MMR Root
        {
            let proof: mmr::MMRProof = {
                let max_index = new_max_height - old_client.headers_mmr_root.min_height;
                let mmr_size = leaf_index_to_mmr_size(u64::from(max_index));
                debug!("check MMR root with size: {mmr_size}, max-index: {max_index}");
                let proof = update.new_headers_mmr_proof().into_iter().collect();
                mmr::MMRProof::new(mmr_size, proof)
            };
            let result = proof
                .verify_incremental(
                    packed_new_client.headers_mmr_root(),
                    self.headers_mmr_root(),
                    digests,
                )
                .map_err(|_| UpdateError::Mmr)?;
            if !result {
                warn!(
                    "failed: verify MMR proof for headers between {} and {new_max_height}",
                    old_client.headers_mmr_root.max_height + 1
                );
                return Err(UpdateError::HeadersMmrProof);
            } else {
                debug!(
                    "passed: verify MMR proof for headers between {} and {new_max_height}",
                    old_client.headers_mmr_root.max_height + 1
                );
            }
        }

        // Check New Client
        if new_client.id != old_client.id {
            error!(
                "failed: new client id has been changed ({} -> {})",
                old_client.id, new_client.id
            );
            return Err(UpdateError::ClientId);
        }
        if new_client.tip_block_hash != new_tip_block_hash {
            error!(
                "failed: new client tip block hash ({:#x}) is incorrect, \
                expect {new_tip_block_hash}",
                new_client.tip_block_hash
            );
            return Err(UpdateError::ClientTipBlockHash);
        }
        if new_client.headers_mmr_root.min_height != old_client.headers_mmr_root.min_height {
            error!(
                "failed: new client min height has been changed ({} -> {})",
                old_client.headers_mmr_root.min_height, new_client.headers_mmr_root.min_height,
            );
            return Err(UpdateError::ClientMinimalHeight);
        }
        if new_client.headers_mmr_root.max_height != new_max_height {
            error!(
                "failed: new client max height ({}) is incorrect, expect {new_max_height}",
                new_client.headers_mmr_root.max_height
            );
            return Err(UpdateError::ClientMaximalHeight);
        }
        let new_target_adjust_info = packed::TargetAdjustInfo::encode(new_info.0, new_info.1);
        if new_client.target_adjust_info.as_slice() != new_target_adjust_info.as_slice() {
            error!(
                "failed: new client's target adjust info is incorrect, \
                expect {new_target_adjust_info:#x} but got {:#x}",
                new_client.target_adjust_info
            );
            return Err(UpdateError::ClientTargetAdjustInfo);
        }

        Ok(())
    }

    /// Verifies whether a transaction is in the chain or not.
    ///
    /// Do the same checks as `self.verify_transaction(..)`,
    /// but require the transaction data as an input argument rather than `Txid`.
    ///
    /// Since the header and the transaction has been recovered from bytes,
    /// so this function return them in order to any possible future usages.
    /// If you don't need them, just ignore them.
    pub fn verify_transaction_data(
        &self,
        tx: &[u8],
        tx_proof: packed::TransactionProofReader,
        confirmations: u32,
    ) -> Result<(core::Header, core::Transaction), VerifyTxError> {
        let tx: core::Transaction =
            deserialize(tx).map_err(|_| VerifyTxError::DecodeTransaction)?;
        let txid = tx.txid();
        let header = self.verify_transaction(txid.as_ref(), tx_proof, confirmations)?;
        Ok((header, tx))
    }

    /// Verifies whether a transaction is in the chain or not.
    ///
    /// Checks:
    /// - Check if the transaction is contained in the provided header (via Merkle proof).
    ///   - In current version, only one transaction could be included in the Merkle proof.
    /// - Check if the header is contained in the Bitcoin chain (via MMR proof).
    /// - Check the confirmation blocks based on the tip header in current SPV client.
    ///   - `0` means skip the check of the confirmation blocks.
    ///
    /// Since the header has been recovered from bytes, so this function return it
    /// in order to any possible future usages.
    /// If you don't need it, just ignore it.
    pub fn verify_transaction(
        &self,
        txid: &[u8; 32],
        tx_proof: packed::TransactionProofReader,
        confirmations: u32,
    ) -> Result<core::Header, VerifyTxError> {
        let height: u32 = tx_proof.height().unpack();
        let min_height = self.headers_mmr_root().min_height().unpack();
        let max_height = self.headers_mmr_root().max_height().unpack();

        // Verify Transaction
        if min_height > height {
            return Err(VerifyTxError::TransactionTooOld);
        }
        if height > max_height {
            return Err(VerifyTxError::TransactionTooNew);
        }
        if confirmations > 0 && max_height - height < confirmations {
            return Err(VerifyTxError::TransactionUnconfirmed);
        }

        // Verify TxOut proof
        let header = {
            let merkle_block: core::MerkleBlock =
                deserialize(tx_proof.transaction_proof().raw_data())
                    .map_err(|_| VerifyTxError::DecodeTxOutProof)?;

            let mut matches: Vec<core::Txid> = vec![];
            let mut indexes: Vec<u32> = vec![];

            merkle_block
                .extract_matches(&mut matches, &mut indexes)
                .map_err(|_| VerifyTxError::TxOutProofIsInvalid)?;

            if matches.len() != indexes.len() {
                return Err(VerifyTxError::TxOutProofIsInvalid);
            }

            let tx_index: u32 = tx_proof.tx_index().unpack();
            indexes
                .into_iter()
                .position(|v| v == tx_index)
                .map(|i| matches[i])
                .ok_or(VerifyTxError::TxOutProofInvalidTxIndex)
                .and_then(|id| {
                    let id_bytes: &[u8; 32] = id.as_ref();
                    if id_bytes == txid {
                        Ok(())
                    } else {
                        Err(VerifyTxError::TxOutProofInvalidTxId)
                    }
                })?;

            merkle_block.header
        };

        // Verify Header MMR proof
        {
            let block_hash = header.block_hash();

            let proof: mmr::MMRProof = {
                let max_index = max_height - min_height;
                let mmr_size = leaf_index_to_mmr_size(u64::from(max_index));
                trace!(
                    "verify MMR proof for header-{height} with \
                    MMR {{ size: {mmr_size}, max-index: {max_index} }}, root: {block_hash:#x}",
                );
                let proof = tx_proof
                    .header_proof()
                    .iter()
                    .map(|r| r.to_entity())
                    .collect::<Vec<_>>();
                mmr::MMRProof::new(mmr_size, proof)
            };
            let digests_with_positions = {
                let index = height - min_height;
                let position = leaf_index_to_pos(u64::from(index));
                trace!(
                    "verify MMR proof for header-{height} with \
                    index: {index}, position: {position}, root: {block_hash:#x}"
                );
                let digest = core::HeaderDigest::new_leaf(height, &header).pack();
                vec![(position, digest)]
            };
            proof
                .verify(self.headers_mmr_root(), digests_with_positions)
                .map_err(|_| VerifyTxError::HeaderMmrProof)?;
        }

        Ok(header)
    }

    /// Compare two chains, which is better.
    pub fn is_better_than(&self, other: &Self) -> bool {
        let self_work = self.headers_mmr_root().partial_chain_work().unpack();
        let other_work = other.headers_mmr_root().partial_chain_work().unpack();
        self_work > other_work
    }
}
