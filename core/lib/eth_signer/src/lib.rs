#[macro_use]
extern crate serde_derive;

use error::SignerError;
use zksync_types::tx::TxEthSignature;
use zksync_types::{Address, H256};

pub use external_signer::{ExternalEthSigner, ExternalSigner};
pub use json_rpc_signer::JsonRpcSigner;
pub use pk_signer::PrivateKeySigner;
pub use raw_ethereum_tx::RawTransaction;

pub mod error;
pub mod external_signer;
pub mod json_rpc_signer;
pub mod pk_signer;
pub mod raw_ethereum_tx;

#[derive(Clone)]
pub enum EthereumSigner {
    PrivateKey(PrivateKeySigner),
    JsonRpc(JsonRpcSigner),
    External(ExternalEthSigner),
}

impl EthereumSigner {
    /// Creates a signer from a private key.
    pub fn from_key(private_key: H256) -> Self {
        let signer = PrivateKeySigner::new(private_key);
        Self::PrivateKey(signer)
    }

    /// Creates a signer with JsonRpcSigner
    /// who does not disclose the private key
    /// while signing the necessary messages and transactions.
    pub fn from_rpc(rpc_signer: JsonRpcSigner) -> Self {
        Self::JsonRpc(rpc_signer)
    }

    /// The sign method calculates an Ethereum specific signature with:
    /// sign(keccak256("\x19Ethereum Signed Message:\n" + len(message) + message))).
    pub async fn sign_message(&self, message: &[u8]) -> Result<TxEthSignature, SignerError> {
        match self {
            EthereumSigner::PrivateKey(pk_signer) => pk_signer.sign_message(message),
            EthereumSigner::JsonRpc(json_rpc_signer) => json_rpc_signer.sign_message(message).await,
            EthereumSigner::External(external_signer) => external_signer.sign_message(message).await,
        }
    }

    /// Signs and returns the RLP-encoded transaction.
    pub async fn sign_transaction(&self, raw_tx: RawTransaction) -> Result<Vec<u8>, SignerError> {
        match self {
            EthereumSigner::PrivateKey(pk_signer) => pk_signer.sign_transaction(raw_tx),
            EthereumSigner::JsonRpc(json_rpc_signer) => {
                json_rpc_signer.sign_transaction(raw_tx).await
            }
            EthereumSigner::External(external_signer) => {
                external_signer.sign_transaction(raw_tx).await
            }
        }
    }

    /// Get Ethereum address.
    pub fn get_address(&self) -> Result<Address, SignerError> {
        match self {
            EthereumSigner::PrivateKey(pk_signer) => pk_signer.address(),
            EthereumSigner::JsonRpc(json_rpc_signer) => Ok(json_rpc_signer.address()?),
            EthereumSigner::External(external_signer) => Ok(external_signer.address()?),
        }
    }
}
