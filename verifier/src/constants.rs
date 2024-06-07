//! Constants.

// Constants for the chain type flag
// Specifically utilizing the two highest bits for chain type identification
pub const FLAG_CHAIN_TYPE_MAINNET: u8 = 0b0000_0000; // for mainnet
pub const FLAG_CHAIN_TYPE_TESTNET: u8 = 0b1000_0000; // for testnet
pub const FLAG_CHAIN_TYPE_SIGNET: u8 = 0b0100_0000; // for signet
