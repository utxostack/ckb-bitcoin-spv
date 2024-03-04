use alloc::borrow::ToOwned;

use bitcoin_hashes::Hash as _;

use crate::types::{core, packed, prelude::*};

macro_rules! impl_conversion_for_entity_unpack {
    ($name:ident) => {
        impl Unpack<core::$name> for packed::$name {
            fn unpack(&self) -> core::$name {
                self.as_reader().unpack()
            }
        }
    };
    ($from:ident, $to:ty) => {
        impl Unpack<$to> for packed::$from {
            fn unpack(&self) -> $to {
                self.as_reader().unpack()
            }
        }
    };
}

//
// Baisc
//

impl<'r> Unpack<u32> for packed::Uint32Reader<'r> {
    fn unpack(&self) -> u32 {
        let mut b = [0u8; 4];
        b.copy_from_slice(self.as_slice());
        u32::from_le_bytes(b)
    }
}
impl_conversion_for_entity_unpack!(Uint32, u32);

impl<'r> Unpack<core::Hash> for packed::HashReader<'r> {
    fn unpack(&self) -> core::Hash {
        let mut b = [0u8; 32];
        b.copy_from_slice(self.as_slice());
        core::Hash::from_byte_array(b)
    }
}
impl_conversion_for_entity_unpack!(Hash);

impl<'r> Unpack<core::Bytes> for packed::BytesReader<'r> {
    fn unpack(&self) -> core::Bytes {
        self.raw_data().to_owned().into()
    }
}
impl_conversion_for_entity_unpack!(Bytes);

//
// Proofs
//

impl<'r> Unpack<core::HeaderDigest> for packed::HeaderDigestReader<'r> {
    fn unpack(&self) -> core::HeaderDigest {
        core::HeaderDigest {
            min_height: self.min_height().unpack(),
            max_height: self.max_height().unpack(),
            children_hash: self.children_hash().unpack(),
        }
    }
}
impl_conversion_for_entity_unpack!(HeaderDigest);

impl<'r> Unpack<core::MmrProof> for packed::MmrProofReader<'r> {
    fn unpack(&self) -> core::MmrProof {
        self.iter().map(|v| v.unpack()).collect()
    }
}
impl_conversion_for_entity_unpack!(MmrProof);

//
// Cells Data
//

impl<'r> Unpack<core::SpvInfo> for packed::SpvInfoReader<'r> {
    fn unpack(&self) -> core::SpvInfo {
        core::SpvInfo {
            tip_client_id: self.tip_client_id().into(),
        }
    }
}
impl_conversion_for_entity_unpack!(SpvInfo);

impl<'r> Unpack<core::SpvClient> for packed::SpvClientReader<'r> {
    fn unpack(&self) -> core::SpvClient {
        core::SpvClient {
            id: self.id().into(),
            tip_block_hash: self.tip_block_hash().unpack(),
            headers_mmr_root: self.headers_mmr_root().unpack(),
            target_adjust_info: self.target_adjust_info().to_entity(),
        }
    }
}
impl_conversion_for_entity_unpack!(SpvClient);

//
// Script Args
//

impl<'r> Unpack<core::SpvTypeArgs> for packed::SpvTypeArgsReader<'r> {
    fn unpack(&self) -> core::SpvTypeArgs {
        core::SpvTypeArgs {
            type_id: self.type_id().unpack(),
            clients_count: self.clients_count().into(),
        }
    }
}
impl_conversion_for_entity_unpack!(SpvTypeArgs);
