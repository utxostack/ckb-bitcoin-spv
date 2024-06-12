use std::fs::read_to_string;

use alloc::format;
use ckb_jsonrpc_types::TransactionView;
use ckb_types::packed::WitnessArgs;
use serde_json::from_str as from_json_str;

use crate::{
    error::UpdateError,
    molecule::prelude::*,
    tests,
    types::{
        core,
        packed::{SpvClient, SpvUpdate},
        prelude::*,
    },
};

// This case shows that:
// - For the signet network, `header.bits` should be the same as `new_info.1`.
// To run this test, use the following command:
// `cargo test --package ckb-bitcoin-spv-verifier --lib -- tests::signet::signet_verify_new_client_error_197568 --exact --show-output`
#[test]
fn signet_verify_new_client_error_197568() {
    let ret = verify_new_client_common(
        "tx-0528-error-check-header-target-adjust-info.json",
        1, // cell_dep_index
    );
    assert!(ret.is_err());
}

// This case shows that:
// - For the signet network, target max should be Target::MAX_ATTAINABLE_SIGNET.
// To run this test, use the following command:
// `cargo test --package ckb-bitcoin-spv-verifier --lib -- tests::signet::signet_verify_new_client_error_header_197567 --exact --show-output`
#[test]
fn signet_verify_new_client_error_header_197567() {
    let ret = verify_new_client_common(
        "tx-0xd663a1dfdfbf9a4824950c44c0d5f5e65f6b1ba4710a0308edecadeaed3ac549.json",
        2, // cell_dep_index
    );
    assert!(ret.is_err());
}

// To run this test, use the following command:
// `cargo test --package ckb-bitcoin-spv-verifier --lib -- tests::signet::signet_verify_new_client_normal --exact --show-output`
#[test]
fn signet_verify_new_client_normal() {
    let ret = verify_new_client_common(
        "tx-0x01d827b049778ffb53532d8263009512a696647bde4acc7f10f39ded14c066ab.json",
        2, // cell_dep_index
    );
    assert!(ret.is_ok());
}

fn verify_new_client_common(tx_file: &str, cell_dep_index: usize) -> Result<(), UpdateError> {
    tests::setup();

    let path = tests::data::find_bin_file("signet", tx_file);
    let tx = read_to_string(path).unwrap();
    let tx: TransactionView = from_json_str(&tx).unwrap();

    let witnesses = tx.inner.witnesses;
    let witness_args = WitnessArgs::from_slice(witnesses[0].as_bytes()).unwrap();
    let spv_update_bin = witness_args.output_type().to_opt().unwrap().raw_data();
    let spv_update = SpvUpdate::from_slice(&spv_update_bin).unwrap();

    let client_bin = tx.inner.outputs_data[1].clone();
    let client = SpvClient::from_slice(client_bin.as_bytes()).unwrap();

    let cell_dep = tx.inner.cell_deps[cell_dep_index].out_point.clone();
    let path =
        tests::data::find_bin_file("signet", format!("tx-0x{}.json", cell_dep.tx_hash).as_str());
    let previous_tx = read_to_string(path).unwrap();
    let previous_tx: TransactionView = from_json_str(&previous_tx).unwrap();
    let cell_dep_data_bin = &previous_tx.inner.outputs_data[cell_dep.index.value() as usize];
    let cell_dep_client = SpvClient::from_slice(cell_dep_data_bin.as_bytes()).unwrap();

    let mut cell_dep_client: core::SpvClient = cell_dep_client.unpack();
    cell_dep_client.id = client.id().into();
    let input_client = cell_dep_client.pack();
    input_client.verify_new_client(&client, spv_update, 64)
}
