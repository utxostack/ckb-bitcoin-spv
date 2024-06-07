//! The utilities for [Bitcoin].
//!
//! [Bitcoin]: https://bitcoin.org/

use bitcoin::{blockdata::constants::DIFFCHANGE_TIMESPAN, pow::Target};
use primitive_types::U256;

use crate::types::core::BitcoinChainType;

/// Calculates the next target.
///
/// N.B. The end time is not the block time of the next 2016-th header.
///
/// Ref:
/// - [What is the Target in Bitcoin?](https://learnmeabitcoin.com/technical/target)
/// - [`CalculateNextWorkRequired(..)` in Bitcoin source code](https://github.com/bitcoin/bitcoin/blob/v26.0/src/pow.cpp#L49)
pub fn calculate_next_target(
    prev_target: Target,
    start_time: u32,
    end_time: u32,
    flags: u8,
) -> Target {
    let expected = DIFFCHANGE_TIMESPAN;
    let actual = {
        let mut actual = end_time - start_time;
        if actual < expected / 4 {
            actual = expected / 4;
        }
        if actual > expected * 4 {
            actual = expected * 4;
        }
        actual
    };

    let le_bytes = {
        let prev_target_le_bytes = prev_target.to_le_bytes();
        let x = U256::from_little_endian(&prev_target_le_bytes);
        trace!("prev-target = {x}");
        let y = x * U256::from(actual);
        trace!("prev-target * {actual} = {y}");
        let z = y / U256::from(expected);
        trace!("{y} / {expected} = {z}");

        let mut le_bytes = [0u8; 32];
        z.to_little_endian(&mut le_bytes);
        le_bytes
    };

    let target = Target::from_le_bytes(le_bytes);
    let max_target = match flags.into() {
        BitcoinChainType::Signet => Target::MAX_ATTAINABLE_SIGNET,
        _ => Target::MAX,
    };
    if target > max_target {
        trace!("fallback to the max target");
        max_target
    } else {
        trace!("use the calculated target");
        target
    }
}
