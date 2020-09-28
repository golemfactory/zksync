// Built-in imports
use std::{fmt, sync::Mutex};
// External uses
use num::BigUint;
use web3::types::H256;
// Workspace uses
use crypto_exports::rand::{thread_rng, Rng};
use models::node::{
    priv_key_from_fs, AccountId, Address, Nonce, TokenId, Transfer, PubKeyHash,
    tx::PackedEthSignature
};

pub use crypto_exports::franklin_crypto::bellman::pairing::bn256::{Bn256 as Engine, Fr};

pub type Fs = <Engine as JubjubEngine>::Fs;


use crypto_exports::franklin_crypto::{
    alt_babyjubjub::fs::FsRepr,
    bellman::pairing::ff::{PrimeField, PrimeFieldRepr},
    eddsa::PrivateKey,
    jubjub::JubjubEngine,
};

use sha2::{Digest, Sha256};

/// Structure used to sign ZKSync transactions, keeps tracks of its nonce internally
pub struct ZksyncAccount {
    pub private_key: PrivateKey<Engine>,
    pub pubkey_hash: PubKeyHash,
    pub address: Address,
    //pub eth_private_key: H256,
    account_id: Mutex<Option<AccountId>>,
    nonce: Mutex<Nonce>,
}

impl fmt::Debug for ZksyncAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // It is OK to disclose the private key contents for a testkit account.
        let mut pk_contents = Vec::new();
        self.private_key
            .write(&mut pk_contents)
            .expect("Failed writing the private key contents");

        f.debug_struct("ZksyncAccount")
            .field("private_key", &pk_contents)
            .field("pubkey_hash", &self.pubkey_hash)
            .field("address", &self.address)
            // .field("eth_private_key", &self.eth_private_key)
            .field("nonce", &self.nonce)
            .finish()
    }
}

impl ZksyncAccount {
    /// Note: probably not secure, use for testing.
    pub fn rand() -> Self {
        let rng = &mut thread_rng();

        let pk = priv_key_from_fs(rng.gen());
        let (eth_pk, eth_address) = {
            let eth_pk = rng.gen::<[u8; 32]>().into();
            let eth_address;
            loop {
                if let Ok(address) = PackedEthSignature::address_from_private_key(&eth_pk) {
                    eth_address = address;
                    break;
                }
            }
            (eth_pk, eth_address)
        };
        Self::new(pk, 0, eth_address, eth_pk)
    }

    pub fn new(
        private_key: PrivateKey<Engine>,
        nonce: Nonce,
        address: Address,
        eth_private_key: H256,
    ) -> Self {
        let pubkey_hash = PubKeyHash::from_privkey(&private_key);
        assert_eq!(
            address,
            PackedEthSignature::address_from_private_key(&eth_private_key)
                .expect("private key is incorrect"),
            "address should correspond to private key"
        );
        Self {
            account_id: Mutex::new(None),
            address,
            private_key,
            pubkey_hash,
            //eth_private_key,
            nonce: Mutex::new(nonce),
        }
    }

    pub fn from_seed(seed: &[u8], address: Address) -> Self {
        let raw_private_key = private_key_from_seed(seed);
        let private_key = read_signing_key(&raw_private_key);
        let pubkey_hash = PubKeyHash::from_privkey(&private_key);
        Self {
            account_id: Mutex::new(None),
            address,
            private_key,
            pubkey_hash,
            //eth_private_key,
            nonce: Mutex::new(0),
        }
    }


    pub fn nonce(&self) -> Nonce {
        let n = self.nonce.lock().unwrap();
        *n
    }

    pub fn set_nonce(&self, new_nonce: Nonce) {
        *self.nonce.lock().unwrap() = new_nonce;
    }

    pub fn set_account_id(&self, account_id: Option<AccountId>) {
        *self.account_id.lock().unwrap() = account_id;
    }

    pub fn get_account_id(&self) -> Option<AccountId> {
        *self.account_id.lock().unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn sign_transfer(
        &self,
        token_id: TokenId,
        token_symbol: &str,
        amount: BigUint,
        fee: BigUint,
        to: &Address,
        nonce: Option<Nonce>,
        increment_nonce: bool,
    ) -> (Transfer, String) {
        let mut stored_nonce = self.nonce.lock().unwrap();
        let transfer = Transfer::new_signed(
            self.account_id
                .lock()
                .unwrap()
                .expect("can't sign tx withoud account id"),
            self.address,
            *to,
            token_id,
            amount,
            fee,
            nonce.unwrap_or_else(|| *stored_nonce),
            &self.private_key,
        )
        .expect("Failed to sign transfer");

        if increment_nonce {
            *stored_nonce += 1;
        }

        let eth_sign_message = transfer.get_ethereum_sign_message(token_symbol, 18);

        (transfer, eth_sign_message)
    }
}


fn private_key_from_seed(seed: &[u8]) -> Vec<u8> {
    if seed.len() < 32 {
        panic!("Seed is too short");
    };

    let sha256_bytes = |input: &[u8]| -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.input(input);
        hasher.result().to_vec()
    };

    let mut effective_seed = sha256_bytes(seed);

    loop {
        let raw_priv_key = sha256_bytes(&effective_seed);
        let mut fs_repr = FsRepr::default();
        fs_repr
            .read_be(&raw_priv_key[..])
            .expect("failed to read raw_priv_key");
        if Fs::from_repr(fs_repr).is_ok() {
            return raw_priv_key;
        } else {
            effective_seed = raw_priv_key;
        }
    }
}

fn read_signing_key(private_key: &[u8]) -> PrivateKey<Engine> {
    let mut fs_repr = FsRepr::default();
    fs_repr
        .read_be(private_key)
        .expect("couldn't read private key repr");
    PrivateKey(Fs::from_repr(fs_repr).expect("couldn't read private key from repr"))
}
