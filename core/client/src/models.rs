use std::collections::HashMap;
// External uses
use num::BigUint;
// Workspace uses
use models::{
    node::{
        AccountId, Address, Nonce, PubKeyHash,
    },
    primitives::{BigUintSerdeAsRadix10Str, BigUintSerdeWrapper},
};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResponseAccountState {
    pub balances: HashMap<String, BigUintSerdeWrapper>,
    pub nonce: Nonce,
    pub pub_key_hash: PubKeyHash,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepositingFunds {
    #[serde(with = "BigUintSerdeAsRadix10Str")]
    amount: BigUint,
    expected_accept_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepositingAccountBalances {
    balances: HashMap<String, DepositingFunds>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfoResp {
    pub address: Address,
    pub id: Option<AccountId>,
    depositing: DepositingAccountBalances,
    pub committed: ResponseAccountState,
    pub verified: ResponseAccountState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    pub block_number: i64,
    pub committed: bool,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfoResp {
    pub executed: bool,
    pub success: Option<bool>,
    pub fail_reason: Option<String>,
    pub block: Option<BlockInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ETHOpInfoResp {
    pub executed: bool,
    pub block: Option<BlockInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ContractAddressResp {
    pub main_contract: String,
    pub gov_contract: String,
}

/// Flattened `PriorityOp` object representing a deposit operation.
/// Used in the `OngoingDepositsResp`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OngoingDeposit {
    received_on_block: u64,
    token_id: u16,
    amount: u128,
    eth_tx_hash: String,
}

/// Information about ongoing deposits for certain recipient address.
///
/// Please note that since this response is based on the events that are
/// currently awaiting confirmations, this information is approximate:
/// blocks on Ethereum can be reverted, and final list of executed deposits
/// can differ from the this estimation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OngoingDepositsResp {
    /// Address for which response is served.
    address: Address,
    /// List of tuples (Eth block number, Deposit operation) of ongoing
    /// deposit operations.
    deposits: Vec<OngoingDeposit>,

    /// Amount of confirmations required for every deposit to be processed.
    confirmations_for_eth_event: u64,

    /// Estimated block number for deposits completions:
    /// all the deposit operations for provided address are expected to be
    /// accepted in the zkSync network upon reaching this blocks.
    ///
    /// Can be `None` if there are no ongoing deposits.
    estimated_deposits_approval_block: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub enum RpcErrorCodes {
    NonceMismatch = 101,
    IncorrectTx = 103,
    FeeTooLow = 104,

    MissingEthSignature = 200,
    EIP1271SignatureVerificationFail = 201,
    IncorrectEthSignature = 202,
    ChangePkNotAuthorized = 203,

    Other = 300,
    AccountCloseDisabled = 301,
    OperationsLimitReached = 302,
}
