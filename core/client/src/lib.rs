#![recursion_limit = "256"]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

pub mod zksync_account;
pub mod wallet;
pub mod rpc_client;
pub mod models;
