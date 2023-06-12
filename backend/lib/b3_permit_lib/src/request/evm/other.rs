use crate::{
    error::RequestError,
    request::result::ExecutionResult,
    request::{request::RequestTrait, result::EvmContractDeployed},
};
use async_trait::async_trait;
use b3_wallet_lib::{
    error::WalletError,
    ledger::evm::{
        evm::EvmSignTrait,
        london::EvmTransaction1559,
        utils::{create_address_from, vec_u8_to_string},
    },
    store::with_ledger,
};
use ic_cdk::export::{candid::CandidType, serde::Deserialize};

// DEPLOY CONTRACT
#[derive(CandidType, Clone, Deserialize, Debug, PartialEq)]
pub struct EvmDeployContract {
    account_id: String,
    chain_id: u64,
    nonce: u64,
    hex_byte_code: Vec<u8>,
    gas_limit: Option<u64>,
    max_fee_per_gas: Option<u64>,
    max_priority_fee_per_gas: Option<u64>,
}

#[async_trait]
impl RequestTrait for EvmDeployContract {
    async fn execute(self) -> Result<ExecutionResult, WalletError> {
        let ledger = with_ledger(&self.account_id, |ledger| ledger.clone())?;

        let public_key = ledger.eth_public_key()?;

        let contract_address = create_address_from(&public_key, self.nonce);

        let data = "0x".to_owned() + &vec_u8_to_string(&self.hex_byte_code);

        // TODO: get default gas limit from user settings
        let gas_limit = self.gas_limit.unwrap_or(0);
        let max_fee_per_gas = self.max_fee_per_gas.unwrap_or(0);
        let max_priority_fee_per_gas = self.max_priority_fee_per_gas.unwrap_or(0);

        let mut transaction = EvmTransaction1559 {
            nonce: self.nonce,
            chain_id: self.chain_id,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            to: "0x".to_string(),
            value: 0,
            data,
            access_list: vec![],
            v: "0x00".to_string(),
            r: "0x00".to_string(),
            s: "0x00".to_string(),
        };

        let raw_tx = transaction.serialized();

        let signature = ledger.sign_with_ecdsa(raw_tx).await?;

        transaction.sign(signature, public_key)?;

        Ok(EvmContractDeployed {
            transaction,
            contract_address,
        }
        .into())
    }

    fn validate_request(&self) -> Result<(), RequestError> {
        with_ledger(&self.account_id, |ledger| {
            if ledger.evm(self.chain_id).is_some() {
                Ok(())
            } else {
                Err(RequestError::ChainIdNotInitialized)
            }
        })?
    }

    fn method_name(&self) -> String {
        "evm_deploy_contract".to_string()
    }
}