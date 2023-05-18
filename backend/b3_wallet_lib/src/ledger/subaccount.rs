use b3_helper::types::{AccountIdentifier, Environment, Subaccount};

use super::{config::EcdsaConfig, types::EcdsaKeyId};

pub trait SubaccountTrait {
    fn account_identifier(&self) -> AccountIdentifier;
    fn environment(&self) -> Environment;
    fn nonce(&self) -> u64;
    fn id(&self) -> String;
    fn derivation_path(&self) -> Vec<Vec<u8>>;
    fn config(&self) -> EcdsaConfig;
    fn key_id(&self) -> EcdsaKeyId;
    fn key_id_with_cycles_and_path(&self) -> (EcdsaKeyId, u64, Vec<Vec<u8>>);
}

impl SubaccountTrait for Subaccount {
    fn account_identifier(&self) -> AccountIdentifier {
        let canister = ic_cdk::id();

        AccountIdentifier::new(&canister, self)
    }

    fn environment(&self) -> Environment {
        match self.0[0] {
            16 => Environment::Staging,
            8 => Environment::Development,
            _ => Environment::Production,
        }
    }

    fn nonce(&self) -> u64 {
        self.0[1..].iter().fold(0, |acc, x| acc + *x as u64)
    }

    fn id(&self) -> String {
        let index = self.nonce();

        let first_byte = self.0[0];

        if first_byte == 0 {
            return "default".to_string();
        }

        let env_str = match first_byte {
            16 => "staging_account",
            8 => "development_account",
            _ => "account",
        };

        [env_str, &index.to_string()].join("_")
    }

    fn derivation_path(&self) -> Vec<Vec<u8>> {
        vec![self.0.to_vec()]
    }

    fn config(&self) -> EcdsaConfig {
        self.environment().into()
    }

    fn key_id(&self) -> EcdsaKeyId {
        self.config().key_id()
    }

    fn key_id_with_cycles_and_path(&self) -> (EcdsaKeyId, u64, Vec<Vec<u8>>) {
        let config = self.config();

        (
            config.key_id(),
            config.sign_cycles(),
            self.derivation_path(),
        )
    }
}

#[cfg(test)]
mod tests {
    use candid::Principal;

    use super::*;

    #[test]
    fn test_initial_subaccount() {
        let subaccount = Subaccount([0; 32]);
        assert_eq!(subaccount.environment(), Environment::Production);
        assert_eq!(subaccount.nonce(), 0);
        assert_eq!(subaccount.id(), "default");
    }

    #[test]
    fn test_subaccount() {
        let subaccount = Subaccount::new(Environment::Production, 1);
        assert_eq!(subaccount.environment(), Environment::Production);
        assert_eq!(subaccount.nonce(), 1);
        assert_eq!(subaccount.id(), "account_1");

        let subaccount = Subaccount::new(Environment::Staging, 1);
        assert_eq!(subaccount.environment(), Environment::Staging);
        assert_eq!(subaccount.nonce(), 1);
        assert_eq!(subaccount.id(), "staging_account_1");

        let subaccount = Subaccount::new(Environment::Development, 1);
        assert_eq!(subaccount.environment(), Environment::Development);
        assert_eq!(subaccount.nonce(), 1);
        assert_eq!(subaccount.id(), "development_account_1");
    }

    #[test]
    fn test_subaccount_from_principal() {
        let principal = Principal::from_text("rno2w-sqaaa-aaaaa-aaacq-cai").unwrap();
        let subaccount = Subaccount::from(principal);
        assert_eq!(subaccount.environment(), Environment::Production);
        assert_eq!(subaccount.nonce(), 7);
        assert_eq!(subaccount.id(), "account_7");
    }

    #[test]
    fn test_subaccount_derivation_path() {
        let subaccount = Subaccount::new(Environment::Production, 0);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]]
        );

        let subaccount = Subaccount::new(Environment::Production, 1);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                32, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]]
        );

        let subaccount = Subaccount::new(Environment::Production, 256);
        assert_eq!(subaccount.environment(), Environment::Production);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                32, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0
            ]]
        );

        let subaccount = Subaccount::new(Environment::Staging, 512);
        assert_eq!(subaccount.environment(), Environment::Staging);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                16, 255, 255, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0
            ]]
        );

        let subaccount = Subaccount::new(Environment::Development, 1024);
        assert_eq!(subaccount.environment(), Environment::Development);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                8, 255, 255, 255, 255, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0
            ]]
        );
    }
}