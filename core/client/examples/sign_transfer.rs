#[macro_use]
extern crate log;

use num::BigUint;
use std::str::FromStr;

use models::node::{Address, tx::PackedEthSignature};
use client::wallet::Wallet;
use client::rpc_client::RpcClient;

#[tokio::main]
async fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_owned());
    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();
    info!("signed transaction example.");
    debug!("set log_level to {}.", std::env::var("RUST_LOG").unwrap());

    let input_to = "d0670f5eA3218bB6A95dD7FAcdCfAC3f19ECAd36";
    let input_token = "GNT";
    let input_amount = "6000000000000000000";
    let input_pk_seed = "6cae8ce3aaf356922b54a0564dbd7075314183e7cfc4fe8478a9bb7b5f7726a31a189146d53997726b9d77f4edf376280cc3609327705b0b175c8423eb6c59261c";

    let provider = RpcClient::new("https://rinkeby-api.zksync.io/jsrpc");

    let pub_key_str = "c38F303B15A34Ee3d21FC4777533b0CA9DdA766F";
    let pub_key_addr = Address::from_str(pub_key_str).unwrap();
    let pk_seed_hex = hex::decode(input_pk_seed).unwrap();
    let wallet = Wallet::from_seed(&pk_seed_hex, pub_key_addr, provider);

    let to = Address::from_str(input_to).unwrap();
    let token = input_token;
    let amount = BigUint::from_str(input_amount).unwrap();

    let (transfer, _eth_sign_message) = wallet.prepare_sync_transfer(
        &to,
        token.to_string(),
        amount,
        None
    ).await;


    let eth_sig_hex = hex::decode("79c2b93604ef97e8ab4cce6bd64b67f9a2cbdef02d7a2cc6bb063acb7e07d1cf77c430759180015161fa8010a178901678a0ffa5f871ac8a4dc8d646421a3f0e1b").expect("failed to decode hex");
    let eth_signature = PackedEthSignature::deserialize_packed(&eth_sig_hex).unwrap();

    let tx_hash = wallet.sync_transfer(transfer, eth_signature).await;

    info!("tx_hash= {}.", hex::encode(tx_hash));
}
