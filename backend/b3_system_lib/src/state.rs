use crate::{
    error::SystemError,
    types::{Controllers, SignerCanisters, State},
    types::{Release, Users},
};
use b3_helper::{
    error::TrapError,
    types::{
        CanisterId, CanisterInstallArg, ControllerId, SignerCanister, SignerCanisterInitArgs,
        UserId,
    },
};
use ic_cdk::api::management_canister::main::CanisterInstallMode;

impl State {
    // user
    pub fn init_user(&mut self, user: UserId) -> Result<SignerCanister, SystemError> {
        let canister = self.users.get(&user);

        if canister.is_some() {
            return Err(SystemError::UserAlreadyExists);
        }

        let signer_canister = SignerCanister::new();

        self.users.insert(user, signer_canister.clone());

        Ok(signer_canister)
    }

    pub fn get_or_init_user(
        &mut self,
        user: UserId,
        opt_canister_id: Option<CanisterId>,
    ) -> Result<SignerCanister, SystemError> {
        if let Some(canister) = self.users.get_mut(&user) {
            return canister
                .get_with_update_rate()
                .map_err(|e| SystemError::SignerCanisterRateError(e.to_string()));
        }

        let signer_canister = if let Some(canister_id) = opt_canister_id {
            SignerCanister::from(canister_id)
        } else {
            SignerCanister::new()
        };

        self.users.insert(user, signer_canister.clone());

        Ok(signer_canister)
    }

    pub fn add_user(&mut self, user: UserId, signer_canister: SignerCanister) {
        self.users.insert(user, signer_canister);
    }

    pub fn remove_user(&mut self, user: &UserId) {
        self.users.remove(user);
    }

    pub fn user_ids(&self) -> Users {
        self.users.keys().cloned().collect()
    }

    pub fn signer_canisters(&self) -> SignerCanisters {
        self.users.values().cloned().collect()
    }

    pub fn number_of_signers(&self) -> usize {
        self.users.len()
    }

    // controller
    pub fn get_controllers(&self) -> Controllers {
        self.controllers.clone()
    }

    pub fn add_controller(&mut self, controller_id: ControllerId) {
        self.controllers.push(controller_id);
    }

    pub fn remove_controller(&mut self, controller_id: ControllerId) {
        self.controllers.retain(|c| c != &controller_id);
    }

    // release
    pub fn latest_release(&self) -> Result<&Release, SystemError> {
        self.releases.last().ok_or(SystemError::ReleaseNotFound)
    }

    pub fn get_latest_install_args(
        &self,
        owner: UserId,
        mode: CanisterInstallMode,
    ) -> Result<CanisterInstallArg, SystemError> {
        let wasm_module = self.latest_release()?.wasm()?;

        let canister_args = SignerCanisterInitArgs { owner };

        let arg = canister_args
            .encode()
            .map_err(|e| SystemError::InstallArgError(e.to_string()))?;

        Ok(CanisterInstallArg {
            wasm_module,
            arg,
            mode,
        })
    }
}