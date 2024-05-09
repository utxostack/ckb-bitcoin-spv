use std::fs::read_to_string;

use ckb_jsonrpc_types::{Either, TransactionView as JsonTransactionView};
use ckb_sdk::CkbRpcClient;
use ckb_types::packed::WitnessArgs;

use serde_json::from_str as from_json_str;

use crate::{
    molecule::prelude::*,
    tests,
    types::{
        core,
        packed::{SpvClient, SpvUpdate},
        prelude::*,
    },
};

const CKB_URL: &str = "https://testnet.ckbapp.dev/";

// This case shows that:
// - For the main network, `header.bits` should be the same as `new_info.1`.
// - But for the test network, this may not be the case.
// To run this test, use the following command:
// `cargo test --package ckb-bitcoin-spv-verifier --lib -- tests::testnet::testnet_verify_new_client_error --exact --show-output`
// Upon running the test, you should expect to see an ERROR output in the log similar to the following:
// [2024-04-30T10:43:45Z ERROR ckb_bitcoin_spv_verifier::types::extension::packed] failed: invalid difficulty for header-2588542, expect 422451157 but got 486604799
#[test]
fn testnet_verify_new_client_error() {
    verify_new_client_common(
        "tx-0422-error-check-header-target-adjust-info.json",
        1, // cell_dep_index
    );
}

#[test]
fn testnet_tx_verify_new_client_normal() {
    verify_new_client_common(
        "tx-0xb5b4a8f31b330d0686fc589b73e8c9c98365a8010bec4719d157671a8c2d7be1.json",
        2, // cell_dep_index
    );
}

fn verify_new_client_common(tx_file: &str, cell_dep_index: usize) {
    tests::setup();

    let path = tests::data::find_bin_file("testnet", tx_file);
    let tx = read_to_string(path).unwrap();
    let tx: JsonTransactionView = from_json_str(&tx).unwrap();

    let witnesses = tx.inner.witnesses;
    let witness_args = WitnessArgs::from_slice(witnesses[0].as_bytes()).unwrap();
    let spv_update_bin = witness_args.output_type().to_opt().unwrap().raw_data();
    let spv_update = SpvUpdate::from_slice(&spv_update_bin).unwrap();

    let client_bin = tx.inner.outputs_data[1].clone();
    let client = SpvClient::from_slice(client_bin.as_bytes()).unwrap();

    let cell_dep = tx.inner.cell_deps[cell_dep_index].out_point.clone();
    let ckb_client = CkbRpcClient::new(CKB_URL);
    let previous_tx = ckb_client
        .get_transaction(cell_dep.tx_hash)
        .unwrap()
        .unwrap();
    let previous_tx = match previous_tx.transaction.unwrap().inner {
        Either::Left(tx) => tx,
        Either::Right(bytes) => serde_json::from_slice(bytes.as_bytes()).unwrap(),
    };
    let cell_dep_data_bin = &previous_tx.inner.outputs_data[cell_dep.index.value() as usize];
    let cell_dep_client = SpvClient::from_slice(cell_dep_data_bin.as_bytes()).unwrap();

    let mut cell_dep_client: core::SpvClient = cell_dep_client.unpack();
    cell_dep_client.id = client.id().into();
    let input_client = cell_dep_client.pack();
    let ret = input_client.verify_new_client(&client, spv_update, 128);

    assert!(ret.is_ok());
}
