use crate::{
    error::SignerError,
    evm_tx::get_evm_transaction,
    ledger::{ledger::Ledger, public_keys::PublicKeys, subaccount::SubaccountTrait},
    signed::SignedTransaction,
};
use b3_helper::types::{Environment, Metadata, Subaccount};
use ic_cdk::export::{candid::CandidType, serde::Deserialize};

#[derive(CandidType, Clone, Deserialize)]
pub struct WalletAccount {
    pub id: String,
    pub name: String,
    pub hidden: bool,
    pub ledger: Ledger,
    pub metadata: Metadata,
}

impl Default for WalletAccount {
    fn default() -> Self {
        WalletAccount {
            id: String::new(),
            name: String::new(),
            hidden: false,
            metadata: Metadata::default(),
            ledger: Ledger::default(),
        }
    }
}

impl From<Subaccount> for WalletAccount {
    fn from(subaccount: Subaccount) -> Self {
        let id = subaccount.id();
        let ledger = subaccount.into();

        WalletAccount {
            id,
            ledger,
            hidden: false,
            name: String::new(),
            metadata: Metadata::default(),
        }
    }
}

impl WalletAccount {
    pub async fn sign_eth_transaction(
        &self,
        hex_raw_tx: Vec<u8>,
        chain_id: u64,
    ) -> Result<SignedTransaction, SignerError> {
        let ecdsa = self.ledger.public_keys.ecdsa()?;

        let mut evm_tx =
            get_evm_transaction(&hex_raw_tx, chain_id).map_err(|e| SignerError::InvalidTx(e))?;

        let message = evm_tx
            .get_message_to_sign()
            .map_err(|e| SignerError::InvalidMsg(e))?;

        if message.len() != 32 {
            return Err(SignerError::InvalidMessageLength);
        }

        let signature = self.ledger.sign_with_ecdsa(message).await?;

        let signed_evm_tx = evm_tx
            .sign(signature, ecdsa)
            .map_err(|e| SignerError::InvalidSignature(e))?;

        let signed_tx = SignedTransaction::new(signed_evm_tx);

        Ok(signed_tx)
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn update_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    pub fn remove_metadata(&mut self, key: &String) {
        self.metadata.remove(key);
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn update_name(&mut self, name: String) -> String {
        self.name = name;

        self.name.clone()
    }

    pub fn hide(&mut self) {
        self.hidden = true;
    }

    pub fn unhide(&mut self) {
        self.hidden = false;
    }

    pub fn public_keys(&self) -> PublicKeys {
        self.ledger.public_keys.clone()
    }

    pub fn environment(&self) -> Environment {
        self.ledger.subaccount.environment()
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }
}
