/*
Example to send a raw transaction prepared by the JS basic scenario from rust.
To reproduce this, add this block to `node_module/zksync/bin/provider.js:80`

```
            console.log('submitTx');
            console.log(tx);
            console.log(signature);
            console.log('submitTx -- DONE');
            if (tx.type === "Transfer")
              process.exit(1);
```

And update the variables in this example to the output.

*/

#[macro_use]
extern crate log;

use num::BigUint;
use std::str::FromStr;

use models::node::{
    tx::{FranklinTx, PackedEthSignature, PackedPublicKey, PackedSignature, TxSignature},
    Transfer, Address
};

use client::rpc_client::RpcClient;

#[tokio::main]
async fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_owned());
    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();
    info!("raw transaction example.");

    let provider = RpcClient::new("https://rinkeby-api.zksync.io/jsrpc");

    let account_id = 149;
    let from = Address::from_str("c38F303B15A34Ee3d21FC4777533b0CA9DdA766F").unwrap();
    let to = Address::from_str("d0670f5eA3218bB6A95dD7FAcdCfAC3f19ECAd36").unwrap();
    let token = 16;
    let amount = BigUint::from_str("6000000000000000000").unwrap();
    let fee = BigUint::from_str("24000000000000000").unwrap();
    let nonce = 3;
    let pk_hex = hex::decode("8d23cd6b165abe0716b423f0d79746372f6162a7fbed76dbfa3d8bcda651ab8b").expect("failed to decode hex");
    let pk_sig = hex::decode("4a178dc074533fec22e6509df474529ba1cba105cc82a3a0a2998c68840d1d92ce0e5d7a3ca0b14a3c3879e71a2e84aa278b53984b897685d128eb84f6126b01").expect("failed to decode hex");
    let signature = TxSignature {
        pub_key: PackedPublicKey::deserialize_packed(&pk_hex).unwrap(),
        signature: PackedSignature::deserialize_packed(&pk_sig).unwrap()
    };

    let transfer = Transfer::new(
        account_id,
        from,
        to,
        token,
        amount,
        fee,
        nonce,
        Some(signature)
    );
    info!("Transfer= {:?}.", transfer);
    let tx = FranklinTx::Transfer(Box::new(transfer));

    let eth_sig_hex = hex::decode("6931f346725ab4ad5a19bfc38a656cf6037447194fef5028ce3620711a67224e3bcab3e18bdb2c8a2a929a43c36f83b39d9229ce93e943733a3918666312937c1b").expect("failed to decode hex");
    let eth_sig = PackedEthSignature::deserialize_packed(&eth_sig_hex).unwrap();
    info!("Signature= {:?}.", eth_sig);
    let tx_hash = provider.send_tx(tx, Some(eth_sig)).await.unwrap();

    info!("tx_hash= {}.", hex::encode(tx_hash));
}
