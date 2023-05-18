use crate::error::WalletError;
use b3_helper::sha2_sha256;
use b3_helper::types::{AccountIdentifier, Subaccount};
use easy_hasher::easy_hasher;
use ic_cdk::export::{candid::CandidType, serde::Deserialize};
use ripemd::{Digest, Ripemd160};
use std::collections::HashMap;

use super::subaccount::SubaccountTrait;
use super::{
    network::{BitcoinNetwork, Network},
    types::{Addresses, Ecdsa},
};

#[derive(CandidType, Deserialize, Clone)]
pub struct PublicKeys {
    pub ecdsa: Option<Ecdsa>,
    pub addresses: Addresses,
    pub identifier: AccountIdentifier,
}

impl Default for PublicKeys {
    fn default() -> Self {
        PublicKeys {
            ecdsa: None,
            addresses: HashMap::new(),
            identifier: AccountIdentifier::default(),
        }
    }
}

impl From<Subaccount> for PublicKeys {
    fn from(subaccount: Subaccount) -> Self {
        let identifier = subaccount.account_identifier();

        let mut addresses = HashMap::new();

        addresses.insert(Network::ICP.to_string(), identifier.to_string());

        PublicKeys {
            ecdsa: None,
            identifier,
            addresses,
        }
    }
}

impl PublicKeys {
    pub fn is_ecdsa_set(&self) -> bool {
        self.ecdsa
            .clone()
            .map(|ecdsa| ecdsa.len() == 33)
            .unwrap_or(false)
    }

    pub fn set_ecdsa(&mut self, ecdsa: Vec<u8>) -> Result<Addresses, WalletError> {
        if self.is_ecdsa_set() {
            return Err(WalletError::EcdsaPublicKeyAlreadySet);
        }

        if ecdsa.len() != 33 {
            return Err(WalletError::InvalidEcdsaPublicKey);
        }

        self.ecdsa = Some(ecdsa);

        self.generate_eth_address(0)?;

        self.generate_btc_address(BitcoinNetwork::Mainnet)?;

        Ok(self.addresses())
    }

    pub fn ecdsa(&self) -> Result<Vec<u8>, WalletError> {
        match &self.ecdsa {
            Some(ecdsa) => Ok(ecdsa.clone()),
            None => Err(WalletError::MissingEcdsaPublicKey),
        }
    }

    pub fn identifier(&self) -> AccountIdentifier {
        self.identifier.clone()
    }

    pub fn addresses(&self) -> HashMap<String, String> {
        self.addresses.clone()
    }

    pub fn generate_address(&mut self, network: Network) -> Result<String, WalletError> {
        match network {
            Network::EVM(chain) => self.generate_eth_address(chain),
            Network::SNS(token) => self.generate_sns_address(token),
            Network::BTC(btc_network) => self.generate_btc_address(btc_network),
            Network::ICP => Ok(self.identifier.to_string()),
        }
    }

    pub fn generate_sns_address(&mut self, token: String) -> Result<String, WalletError> {
        let address = self.identifier.to_string();

        self.addresses
            .insert(Network::SNS(token).to_string(), address.clone());

        Ok(address)
    }

    pub fn generate_eth_address(&mut self, chain: u64) -> Result<String, WalletError> {
        let ecdsa = self.ecdsa()?;

        let pub_key_arr: [u8; 33] = ecdsa[..].try_into().unwrap();

        let pub_key = libsecp256k1::PublicKey::parse_compressed(&pub_key_arr)
            .map_err(|e| WalletError::GenerateError(e.to_string()))?
            .serialize();

        let keccak256 = easy_hasher::raw_keccak256(pub_key[1..].to_vec());
        let keccak256_hex = keccak256.to_hex_string();
        let address: String = "0x".to_owned() + &keccak256_hex[24..];

        self.addresses
            .insert(Network::EVM(chain).to_string(), address.clone());

        Ok(address)
    }

    pub fn generate_btc_address(
        &mut self,
        bitcoin_network: BitcoinNetwork,
    ) -> Result<String, WalletError> {
        let bytes = self.ecdsa()?;

        let mut hasher = Ripemd160::new();
        hasher.update(bytes);
        let result = hasher.finalize();

        let prefix = match bitcoin_network {
            BitcoinNetwork::Mainnet => 0x00,
            _ => 0x6f,
        };

        let mut data_with_prefix = vec![prefix];
        data_with_prefix.extend(result);

        let checksum = &sha2_sha256(&sha2_sha256(&data_with_prefix.clone()))[..4];

        let mut full_address = data_with_prefix;
        full_address.extend(checksum);

        let address: String = bs58::encode(full_address).into_string();

        self.addresses
            .insert(bitcoin_network.to_string(), address.clone());

        Ok(address)
    }
}