// Built-in imports

// External uses
use num::BigUint;
// Workspace uses
pub use models::node::{
    AccountId, Address, TokenId
};
pub use models::node::tx::{FranklinTx, TxHash, PackedEthSignature};

use crate::models::AccountInfoResp;
use crate::rpc_client::RpcClient;
use crate::zksync_account::ZksyncAccount;


pub static ETH_SIGN_MESSAGE: &str = "Access zkSync account.\n\nOnly sign this message for a trusted client!";
pub static ETH_SIGN_POSTFIX: &str = "\nChain ID: {}.";

pub enum BalanceState {
    Committed,
    Verified
}

pub struct Wallet {
    cached_address: Address,
    pub provider: RpcClient,
    // eth_acc: Option<EthereumAccount<Http>>,
    pub sync_acc : Option<ZksyncAccount>,
}

impl Wallet {
    pub fn from_public_address(address: Address, provider: RpcClient) -> Self {
        debug!("Make read-only wallet from address={}", address);
        Wallet {
            cached_address: address,
            provider,
            sync_acc: None
        }
    }

    pub fn from_seed(seed: Vec<u8>, address : Address, provider: RpcClient) -> Self {
        debug!("Make wallet from seed={}", address);
        let sync_acc = ZksyncAccount::from_seed(seed.as_ref(), address);
        Self {
            cached_address: address,
            provider,
            sync_acc: Some(sync_acc)
        }
    }

    pub async fn prepare_sync_transfer(
        &self,
        to: &Address,
        token_symbol: String,
        amount: BigUint,
        fee: Option<BigUint>
    ) -> (FranklinTx, String) {
        let sync_acc = self.sync_acc.as_ref().unwrap();
        let account_id = self.get_account_id().await;
        sync_acc.set_account_id(Some(account_id));
        sync_acc.set_nonce(self.get_nonce().await);


        let token_id = self.resolve_token_id(&token_symbol).await.unwrap();
        info!("token_id= {:?}.", token_id);
        let fee: BigUint = fee.unwrap_or(
            self.provider.get_tx_fee("Transfer", *to, &token_symbol).await.unwrap());
        info!("fee= {:?}.", fee);

        let (transfer, eth_sign_message) = sync_acc.sign_transfer(token_id, &token_symbol, amount, fee, to, None, true);
        info!("Transfer= {:?}.", transfer);
        let tx = FranklinTx::Transfer(Box::new(transfer));
        (tx, eth_sign_message)
    }

    pub async fn sync_transfer(&self, tx: FranklinTx, eth_signature: PackedEthSignature) -> TxHash {
        self.provider.send_tx(tx, Some(eth_signature)).await.unwrap()
    }

    async fn resolve_token_id(&self, token_symbol : &str) -> Option<TokenId> {
        let tokens = self.provider.get_tokens().await.unwrap();
        debug!("tokens= {:?}.", tokens);
        Some(tokens.get(token_symbol)?.id)
    }

    async fn _get_state(&self) -> AccountInfoResp {
        debug!("Query account state={}", self.cached_address);
        self.provider.account_state_info(self.cached_address).await.unwrap()
    }

    pub async fn get_account_id(&self) -> AccountId {
        debug!("get_account_id {}", self.cached_address);
        let account_state: AccountInfoResp = self._get_state().await;
        account_state.id.unwrap()
    }

    pub async fn get_nonce(&self) -> u32 {
        debug!("get_nonce {}", self.cached_address);
        let account_state: AccountInfoResp = self._get_state().await;
        account_state.committed.nonce
    }

    pub async fn get_balance(&self, token: &str, state: BalanceState) -> BigUint {
        let account_state: AccountInfoResp = self._get_state().await;
        debug!("Account state fetched.");
        let balances = match state {
            BalanceState::Committed => account_state.committed.balances,
            BalanceState::Verified => account_state.verified.balances
        };
        debug!("Raw balance = {:?}.", balances);
        balances.get(token).cloned().unwrap().0
    }
}
