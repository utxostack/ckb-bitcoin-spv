use bitcoin::{
    blockdata::constants::DIFFCHANGE_INTERVAL,
    pow::{CompactTarget, Target},
};

use crate::{tests, types::core, utilities::bitcoin::calculate_next_target};

#[test]
fn main_chain_targets() {
    tests::setup();

    let mut prev_height = 0;
    let mut start_time = 0;
    let mut next_bits = CompactTarget::default();
    let interval = DIFFCHANGE_INTERVAL;
    for bin_file in tests::data::find_bin_files("main-chain/headers/diff-change", "") {
        log::info!("process file {}", bin_file.display());

        let header: core::Header = tests::utilities::decode_from_bin_file(&bin_file);

        let file_stem = bin_file.file_stem().unwrap().to_str().unwrap();
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

        if height == 0 {
            log::debug!(">>> initialize data with header-{height}");
            prev_height = height;
            start_time = header.time;
            next_bits = header.bits;
            continue;
        }

        assert_eq!(next_bits, header.bits);

        match (height + 1) % interval {
            0 => {
                assert!(prev_height + interval - 1 == height);
                let next_target = calculate_next_target(curr_target, start_time, header.time);
                log::info!(">>> calculated new target  {next_target:#x}");
                next_bits = next_target.to_compact_lossy();
                let next_target: Target = next_bits.into();
                log::info!(">>> after definition lossy {next_target:#x}");
            }
            1 => {
                assert!(prev_height + 1 == height);
                start_time = header.time;
            }
            remained => {
                panic!(
                    "header-{height} is invalid (remained {remained}), \
                    only `{interval} * N - 1` and `{interval} * N` are allowed"
                );
            }
        }

        prev_height = height;
    }
}
