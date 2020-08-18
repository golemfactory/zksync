use num::BigUint;
use web3::types::{Address};

use crate::rpc_client::RpcClient;

pub enum BalanceState {
    Committed,
    Verified
}

pub struct Wallet {
    cached_address: Address,
    pub provider: RpcClient
}

impl Wallet {
    pub fn from_public_address(address: Address, provider: RpcClient) -> Self {
        debug!("Make read-only wallet from address={}", address);
        Wallet {
            cached_address: address,
            provider
        }
    }

    pub async fn get_balance(&self, token: &str, state: BalanceState) -> BigUint {
        debug!("Query account state={}", self.cached_address);
        let account_state = self.provider.account_state_info(self.cached_address).await.unwrap();
        debug!("Account state fetched.");
        let balances = match state {
            BalanceState::Committed => account_state.committed.balances,
            BalanceState::Verified => account_state.verified.balances
        };
        debug!("Raw balance = {:?}.", balances);
        balances.get(token).cloned().unwrap_or_default().0
    }
}
