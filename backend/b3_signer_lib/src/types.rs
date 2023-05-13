use crate::{account::SignerAccount, allowance::SignerAllowance, request::EvmSignRequest};
use b3_helper::types::CanisterId;
use candid::CandidType;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

pub type Metadata = HashMap<String, String>;

pub type Accounts = BTreeMap<String, SignerAccount>;

pub type CanisterAllowances = HashMap<CanisterId, SignerAllowance>;

pub type CanisterRequests = HashMap<CanisterId, EvmSignRequest>;

#[derive(CandidType, Deserialize)]
pub struct SignerAllowanceArgs {
    pub limit: Option<u8>,
    pub metadata: Metadata,
    pub expires_at: Option<u64>,
}

#[derive(CandidType, Default, Deserialize)]
pub enum TransactionStatus {
    #[default]
    Pending,
    Success,
    Failed,
}

#[derive(CandidType, Deserialize)]
pub struct AccountsStatus {
    pub dev_counter: u64,
    pub prod_counter: u64,
    pub stag_counter: u64,
}