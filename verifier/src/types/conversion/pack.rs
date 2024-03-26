use alloc::vec::Vec;

use bitcoin::consensus::serialize;
use primitive_types::U256;

use crate::types::{bytes::Bytes, core, packed, prelude::*};

//
// Baisc
//

impl Pack<packed::Uint32> for u32 {
    fn pack(&self) -> packed::Uint32 {
        let le = self.to_le_bytes();
        packed::Uint32::new_unchecked(Bytes::from(le.to_vec()))
    }
}

impl Pack<packed::Uint256> for U256 {
    fn pack(&self) -> packed::Uint256 {
        let mut le = [0u8; 32];
        self.to_little_endian(&mut le);
        packed::Uint256::new_unchecked(Bytes::from(le.to_vec()))
    }
}

impl Pack<packed::Hash> for core::Hash {
    fn pack(&self) -> packed::Hash {
        let array = self.to_byte_array();
        packed::Hash::new_unchecked(Bytes::from(array.to_vec()))
    }
}

impl Pack<packed::Header> for core::Header {
    fn pack(&self) -> packed::Header {
        let vec = serialize(self);
        let data = molecule::bytes::Bytes::from(vec);
        packed::Header::new_unchecked(data)
    }
}

impl Pack<packed::HeaderVec> for Vec<core::Header> {
    fn pack(&self) -> packed::HeaderVec {
        packed::HeaderVec::new_builder()
            .set(self.iter().map(|v| v.pack()).collect())
            .build()
    }
}

impl Pack<packed::Bytes> for core::Bytes {
    fn pack(&self) -> packed::Bytes {
        let len = self.len();
        let mut vec: Vec<u8> = Vec::with_capacity(molecule::NUMBER_SIZE + len);
        let len_bytes = molecule::pack_number(len as molecule::Number);
        vec.extend_from_slice(&len_bytes);
        vec.extend_from_slice(self);
        let data = molecule::bytes::Bytes::from(vec);
        packed::Bytes::new_unchecked(data)
    }
}

//
// Proofs
//

impl Pack<packed::HeaderDigest> for core::HeaderDigest {
    fn pack(&self) -> packed::HeaderDigest {
        packed::HeaderDigest::new_builder()
            .min_height(self.min_height.pack())
            .max_height(self.max_height.pack())
            .partial_chain_work(self.partial_chain_work.pack())
            .children_hash(self.children_hash.pack())
            .build()
    }
}

impl Pack<packed::MmrProof> for core::MmrProof {
    fn pack(&self) -> packed::MmrProof {
        packed::MmrProof::new_builder()
            .set(self.iter().map(|v| v.pack()).collect())
            .build()
    }
}

//
// Cells Data
//

impl Pack<packed::SpvInfo> for core::SpvInfo {
    fn pack(&self) -> packed::SpvInfo {
        packed::SpvInfo::new_builder()
            .tip_client_id(self.tip_client_id.into())
            .build()
    }
}

impl Pack<packed::SpvClient> for core::SpvClient {
    fn pack(&self) -> packed::SpvClient {
        packed::SpvClient::new_builder()
            .id(self.id.into())
            .tip_block_hash(self.tip_block_hash.pack())
            .headers_mmr_root(self.headers_mmr_root.pack())
            .target_adjust_info(self.target_adjust_info.clone())
            .build()
    }
}

//
// Script Args
//

impl Pack<packed::SpvTypeArgs> for core::SpvTypeArgs {
    fn pack(&self) -> packed::SpvTypeArgs {
        packed::SpvTypeArgs::new_builder()
            .type_id(self.type_id.pack())
            .clients_count(self.clients_count.into())
            .flags(self.flags.into())
            .build()
    }
}
