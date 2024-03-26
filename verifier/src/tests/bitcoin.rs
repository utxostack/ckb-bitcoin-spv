use std::format;

use bitcoin::{
    blockdata::constants::DIFFCHANGE_INTERVAL,
    pow::{CompactTarget, Target},
};
use primitive_types::U256;

use crate::{tests, types::core, utilities::bitcoin::calculate_next_target};

const DIFF_CHANGE_HEADERS_DIR: &str = "main-chain/headers/diff-change";
const CHAINWORK_BY_HEIGHTS: &str = "main-chain/chainwork";

#[test]
fn main_chain_targets_and_chainwork() {
    tests::setup();

    let mut prev_height = 0;
    let mut start_time = 0;
    let mut next_bits = CompactTarget::default();
    let mut expected_chainwork = U256::zero();
    let interval = DIFFCHANGE_INTERVAL;
    for header_file in tests::data::find_bin_files(DIFF_CHANGE_HEADERS_DIR, "") {
        log::info!("process file {}", header_file.display());

        let header: core::Header = tests::utilities::decode_from_bin_file(&header_file);

        let file_stem = header_file.file_stem().unwrap().to_str().unwrap();
        let height: u32 = file_stem.parse().unwrap();

        let next_target: Target = next_bits.into();
        let curr_target: Target = header.bits.into();
        log::trace!(
            ">>> [ cached] height {prev_height:07}, start-time {start_time}, target {next_target:#x}"
        );
        log::trace!(
            ">>> [current] height {height:07},       time {}, target {curr_target:#x}",
            header.time
        );

        let blockwork = U256::from_little_endian(&curr_target.to_work().to_le_bytes());

        if height == 0 {
            log::debug!(">>> initialize data with header#{height:07}");
            start_time = header.time;
            next_bits = header.bits;
            expected_chainwork = blockwork;
        } else {
            assert_eq!(next_bits, header.bits);

            match (height + 1) % interval {
                0 => {
                    assert!(prev_height + interval - 1 == height);
                    let next_target = calculate_next_target(curr_target, start_time, header.time);
                    log::info!(">>> calculated new target  {next_target:#x}");
                    next_bits = next_target.to_compact_lossy();
                    let next_target: Target = next_bits.into();
                    log::info!(">>> after definition lossy {next_target:#x}");
                    expected_chainwork += blockwork * (interval - 1);
                }
                1 => {
                    assert!(prev_height + 1 == height);
                    start_time = header.time;
                    expected_chainwork += blockwork;
                }
                remained => {
                    panic!(
                        "for current test, header-{height} is invalid (remained {remained}), \
                         only `{interval} * N - 1` and `{interval} * N` are allowed"
                    );
                }
            }
        }
        prev_height = height;

        log::trace!(">>> height {height:07}, block-work {blockwork:#066x}");

        log::trace!(">>> height {height:07}, chain-work {expected_chainwork:#066x} (expected)");
        let actual_chainwork = {
            let filename = format!("{height:07}.bin");
            let file = tests::data::find_bin_file(CHAINWORK_BY_HEIGHTS, &filename);
            let bytes = tests::utilities::load_from_bin_file(&file);
            U256::from_big_endian(&bytes)
        };
        log::trace!(">>> height {height:07}, chain-work {actual_chainwork:#066x} (actual)");

        assert_eq!(expected_chainwork, actual_chainwork);
    }
}
