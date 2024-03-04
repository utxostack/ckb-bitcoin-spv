use std::{format, fs::File, io::Read as _};

use bitcoin::consensus::serialize;
use ckb_bitcoin_spv_verifier::types::{core, packed, prelude::*};

use crate::{tests, utilities, BlockProofGenerator, DummyService};

fn test_spv_client(case_headers: &str, case_txoutproofs: &str, case_blocks: &str) {
    tests::setup();

    let headers_path = format!("main-chain/headers/continuous/{case_headers}");
    let txoutproofs_path = format!("main-chain/txoutproof/{case_txoutproofs}");
    let blocks_path = format!("main-chain/blocks/continuous/{case_blocks}");

    // Bootstrap
    let mut header_bins_iter = tests::data::find_bin_files(&headers_path, "").into_iter();
    let mut service = {
        let header_bin = header_bins_iter.next().unwrap();
        let header: core::Header = utilities::decode_from_bin_file(&header_bin).unwrap();
        let height: u32 = header_bin
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        log::trace!("process header-{height} from file {}", header_bin.display());
        let bootstrap = packed::SpvBootstrap::new_builder()
            .height(height.pack())
            .header(header.pack())
            .build();
        let expected_client = bootstrap
            .initialize_spv_client()
            .map_err(|err| err as i8)
            .unwrap()
            .pack();
        let service = DummyService::bootstrap(height, header).unwrap();
        let actual_client: packed::SpvClient = service.tip_client().pack();
        assert_eq!(expected_client.as_slice(), actual_client.as_slice());
        service
    };

    // Update
    let mut old_client: packed::SpvClient = service.tip_client().pack();
    for header_bin in header_bins_iter {
        let header: core::Header = utilities::decode_from_bin_file(&header_bin).unwrap();
        let height: u32 = header_bin
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        log::trace!("process header-{height} from file {}", header_bin.display());

        let update = service.update(header).unwrap();
        let new_client: packed::SpvClient = service.tip_client().pack();

        old_client
            .verify_new_client(&new_client, update)
            .map_err(|err| err as i8)
            .unwrap();
        old_client = new_client;
    }

    // Verify Tx
    let tip_client: packed::SpvClient = service.tip_client().pack();
    for bin_file in tests::data::find_bin_files(&txoutproofs_path, "") {
        log::trace!("process txoutproof from file {}", bin_file.display());

        let actual = File::open(&bin_file)
            .and_then(|mut file| {
                let mut data = Vec::new();
                file.read_to_end(&mut data).map(|_| data)
            })
            .unwrap();
        let _: core::MerkleBlock =
            utilities::decode_from_slice(&actual).expect("check binary data");

        let file_stem = bin_file.file_stem().unwrap().to_str().unwrap();
        let (height, tx_index) = if let Some((height_str, indexes_str)) = file_stem.split_once('-')
        {
            let height: u32 = height_str.parse().unwrap();
            let indexes = indexes_str
                .split('_')
                .filter(|s| !s.is_empty())
                .map(|s| {
                    s.parse()
                        .map_err(|err| format!("failed to parse \"{s}\" since {err}"))
                })
                .collect::<Result<Vec<u32>, _>>()
                .unwrap();
            if indexes.len() > 1 {
                log::warn!("TODO with current APIs, only ONE tx is allowed each time");
                continue;
            }
            (height, indexes[0])
        } else {
            panic!("invalid txoutproof file stem \"{file_stem}\"");
        };

        let header_proof = service.generate_header_proof(height).unwrap().unwrap();
        let tx_proof = packed::TransactionProof::new_builder()
            .tx_index(tx_index.pack())
            .height(height.pack())
            .transaction_proof(core::Bytes::from(actual).pack())
            .header_proof(header_proof.pack())
            .build();

        let block_filename = format!("{height:07}.bin");
        let block_file = tests::data::find_bin_file(&blocks_path, &block_filename);
        let bpg = BlockProofGenerator::from_bin_file(&block_file).unwrap();
        let tx = bpg.get_transaction(tx_index as usize).unwrap();
        let tx_bytes = serialize(tx);

        let _ = tip_client
            .verify_transaction(&tx_bytes, tx_proof.as_reader())
            .map_err(|err| err as i8)
            .unwrap();
    }
}

#[test]
fn spv_client_case_1() {
    test_spv_client(
        "case-0822528_0830592",
        "case-0830000",
        "case-0830000_0830000",
    );
}
