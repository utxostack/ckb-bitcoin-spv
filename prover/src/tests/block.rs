use std::{fs::File, io::Read as _};

use bitcoin::merkle_tree::MerkleBlock;

use crate::{tests, utilities, BlockProofGenerator};

fn test_new_generator(case: &str) {
    tests::setup();
    let full_dir_path = format!("main-chain/blocks/continuous/{case}");
    for bin_file in tests::data::find_bin_files(&full_dir_path, "") {
        log::trace!("process file {}", bin_file.display());
        assert!(BlockProofGenerator::from_bin_file(&bin_file).is_ok());
    }
}

#[test]
fn new_generator_case_1() {
    test_new_generator("case-0831328_0831335");
}

fn test_generate_txoutproof(case_txoutproofs: &str, case_blocks: &str) {
    tests::setup();

    let txoutproofs_path = format!("main-chain/txoutproof/{case_txoutproofs}");
    let blocks_path = format!("main-chain/blocks/continuous/{case_blocks}");

    for bin_file in tests::data::find_bin_files(&txoutproofs_path, "") {
        log::trace!("process file {}", bin_file.display());

        let actual = File::open(&bin_file)
            .and_then(|mut file| {
                let mut data = Vec::new();
                file.read_to_end(&mut data).map(|_| data)
            })
            .unwrap();
        let _: MerkleBlock = utilities::decode_from_slice(&actual).expect("check binary data");

        let file_stem = bin_file.file_stem().unwrap().to_str().unwrap();
        let (height, indexes) = if let Some((height_str, indexes_str)) = file_stem.split_once('-') {
            let height: u64 = height_str.parse().unwrap();
            let indexes = indexes_str
                .split('_')
                .filter(|s| !s.is_empty())
                .map(|s| {
                    s.parse()
                        .map_err(|err| format!("failed to parse \"{s}\" since {err}"))
                })
                .collect::<Result<Vec<u32>, _>>()
                .unwrap();
            log::trace!(">>> proof in block {height}, for txs {indexes:?}");
            (height, indexes)
        } else {
            panic!("invalid txoutproof file stem \"{file_stem}\"");
        };

        let block_filename = format!("{height:07}.bin");
        let block_file = tests::data::find_bin_file(&blocks_path, &block_filename);
        let bpg = BlockProofGenerator::from_bin_file(&block_file).unwrap();

        let expected = bpg.generate_txoutproof_via_indexes(&indexes).unwrap();

        assert_eq!(expected, actual);
    }
}

#[test]
fn generate_txoutproof_case_1() {
    test_generate_txoutproof("case-0831332", "case-0831328_0831335");
}
