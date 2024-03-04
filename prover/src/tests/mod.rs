use env_logger::{Builder, Target};
use log::LevelFilter;

mod block;
mod service;

pub(crate) mod data;

pub(crate) fn setup() {
    let _ = Builder::new()
        .filter_module("ckb_bitcoin_spv", LevelFilter::Trace)
        .target(Target::Stdout)
        .is_test(true)
        .try_init();
    println!();
}
