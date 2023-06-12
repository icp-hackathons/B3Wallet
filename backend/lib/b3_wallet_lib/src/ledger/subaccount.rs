use super::{config::EcdsaConfig, types::EcdsaKeyId};
use b3_helper_lib::subaccount::Subaccount;

pub trait SubaccountTrait {
    fn derivation_path(&self) -> Vec<Vec<u8>>;
    fn config(&self) -> EcdsaConfig;
    fn key_id(&self) -> EcdsaKeyId;
    fn key_id_with_cycles_and_path(&self) -> (EcdsaKeyId, u64, Vec<Vec<u8>>);
}

impl SubaccountTrait for Subaccount {
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
    use b3_helper_lib::environment::Environment;
    use candid::Principal;

    use super::*;

    const TEST_PRINCIPAL: Principal = Principal::from_slice(&[
        0, 0, 0, 0, 0, 0, 0, 7, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ]);

    #[test]
    fn test_initial_subaccount() {
        let subaccount = Subaccount::default();
        assert_eq!(subaccount.environment(), Environment::Production);
        assert_eq!(subaccount.nonce(), 0);
        assert_eq!(subaccount.name(), "Default");
        assert_eq!(subaccount.id(), "default");

        let identifier = subaccount.account_identifier(TEST_PRINCIPAL);

        println!("{:?}", identifier.to_string());
    }

    #[test]
    fn test_subaccount() {
        let subaccount = Subaccount::new(Environment::Production, 1);
        println!("{:?}", subaccount);
        assert_eq!(subaccount.environment(), Environment::Production);
        assert_eq!(subaccount.nonce(), 1);
        assert_eq!(subaccount.name(), "Account 2");
        assert_eq!(subaccount.id(), "account_1");

        let subaccount = Subaccount::new(Environment::Staging, 1);
        assert_eq!(subaccount.environment(), Environment::Staging);
        assert_eq!(subaccount.nonce(), 1);
        assert_eq!(subaccount.name(), "Staging Account 2");
        assert_eq!(subaccount.id(), "staging_account_1");

        let subaccount = Subaccount::new(Environment::Development, 1);
        assert_eq!(subaccount.environment(), Environment::Development);
        assert_eq!(subaccount.nonce(), 1);
        assert_eq!(subaccount.name(), "Development Account 2");
        assert_eq!(subaccount.id(), "development_account_1");
    }

    #[test]
    fn test_subaccount_from_principal() {
        let subaccount = Subaccount::from(TEST_PRINCIPAL);

        println!("{:?}", subaccount);
        assert_eq!(subaccount.environment(), Environment::Production);
        assert_eq!(subaccount.nonce(), 0);
        assert_eq!(subaccount.name(), "Principal");
        assert_eq!(subaccount.id(), "principal");
    }

    #[test]
    fn test_subaccount_derivation_path() {
        let subaccount = Subaccount::new(Environment::Production, 0);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ]]
        );
        assert_eq!(subaccount.id(), "default");
        assert_eq!(subaccount.name(), "Default");

        let subaccount = Subaccount::new(Environment::Production, 1);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 1
            ]]
        );
        assert_eq!(subaccount.id(), "account_1");
        assert_eq!(subaccount.name(), "Account 2");

        let subaccount = Subaccount::new(Environment::Production, 255);
        assert_eq!(subaccount.environment(), Environment::Production);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 255
            ]]
        );
        assert_eq!(subaccount.id(), "account_255");
        assert_eq!(subaccount.name(), "Account 256");

        let subaccount = Subaccount::new(Environment::Staging, 512);
        assert_eq!(subaccount.environment(), Environment::Staging);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 170, 0, 0, 0,
                0, 0, 0, 2, 0
            ]]
        );
        assert_eq!(subaccount.id(), "staging_account_512");
        assert_eq!(subaccount.name(), "Staging Account 513");

        let subaccount = Subaccount::new(Environment::Development, 1024);
        assert_eq!(subaccount.environment(), Environment::Development);
        assert_eq!(
            subaccount.derivation_path(),
            vec![vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 0, 0,
                0, 0, 0, 4, 0
            ]]
        );
        assert_eq!(subaccount.id(), "development_account_1024");
        assert_eq!(subaccount.name(), "Development Account 1025");
    }
}