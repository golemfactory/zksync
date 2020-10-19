use crate::raw_ethereum_tx::RawTransaction;
use crate::SignerError;

use async_trait::async_trait;

use zksync_types::tx::TxEthSignature;
use zksync_types::Address;

#[async_trait]
pub trait ExternalSigner {
    /// Get Ethereum address that matches the private key.
    async fn get_address(&self) -> Result<Address, SignerError>;

    /// The sign method calculates an Ethereum specific signature with:
    /// sign(keccak256("\x19Ethereum Signed Message:\n" + len(message) + message))).
    async fn sign_message(&self, message: &[u8]) -> Result<TxEthSignature, SignerError>;

    /// Signs and returns the RLP-encoded transaction.
    async fn sign_transaction(&self, raw_tx: RawTransaction) -> Result<Vec<u8>, SignerError>;

    fn box_clone(&self) -> Box<dyn ExternalSigner>;
}

impl Clone for Box<dyn ExternalSigner>
{
    fn clone(&self) -> Box<dyn ExternalSigner> {
        self.box_clone()
    }
}


#[derive(Clone)]
pub struct ExternalEthSigner {
    eth_address: Option<Address>,
    eth_signer: Box<dyn ExternalSigner>,
}

impl ExternalEthSigner {
    pub async fn new(eth_signer: Box<dyn ExternalSigner>) -> Self {
        let eth_address = match eth_signer.get_address().await {
            Ok(addr) => Some(addr),
            _ => None
        };
        Self {
            eth_address,
            eth_signer
        }
    }

    /// Get Ethereum address that matches the private key.
    pub fn address(&self) -> Result<Address, SignerError> {
        self.eth_address.ok_or(SignerError::DefineAddress)
    }

    /// The sign method calculates an Ethereum specific signature with:
    /// sign(keccak256("\x19Ethereum Signed Message:\n" + len(message) + message))).
    pub async fn sign_message(&self, message: &[u8]) -> Result<TxEthSignature, SignerError> {
        self.eth_signer.sign_message(message).await
    }

    /// Signs and returns the RLP-encoded transaction.
    pub async fn sign_transaction(&self, raw_tx: RawTransaction) -> Result<Vec<u8>, SignerError> {
        self.eth_signer.sign_transaction(raw_tx).await
    }
}
