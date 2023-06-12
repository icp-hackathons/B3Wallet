mod account;
mod status;
mod wasm;

use b3_helper_lib::{
    owner::{with_owner, with_owner_mut},
    types::{SignerId, WalletCanisterInitArgs},
    wasm::with_wasm_mut,
};
use b3_wallet_lib::{
    state::WalletState,
    store::{with_wallet, with_wallet_mut},
};
use ic_cdk::{api::call::arg_data, export::candid::candid_method, init, post_upgrade, pre_upgrade};

#[init]
#[candid_method(init)]
pub fn init() {
    let (call_arg,) = arg_data::<(Option<WalletCanisterInitArgs>,)>();

    if let Some(args) = call_arg {
        with_owner_mut(|owner| *owner = args.owner_id);
    } else {
        with_owner_mut(|owner| *owner = ic_cdk::caller());
    }

    with_wallet_mut(|state| state.init_wallet());
}

#[pre_upgrade]
pub fn pre_upgrade() {
    with_wasm_mut(|wasm| wasm.unload());

    let owner = with_owner(|o| o.clone());
    let state = with_wallet(|s| s.clone());

    ic_cdk::storage::stable_save((state, owner)).unwrap();
}

#[post_upgrade]
pub fn post_upgrade() {
    let (state_prev, sign_prev): (WalletState, SignerId) =
        ic_cdk::storage::stable_restore().unwrap();

    with_wallet_mut(|state| *state = state_prev);

    with_owner_mut(|owner| *owner = sign_prev);
}

#[cfg(test)]
mod tests {
    use b3_helper_lib::environment::Environment;
    use b3_helper_lib::tokens::Tokens;
    use b3_helper_lib::types::*;
    use b3_wallet_lib::account::WalletAccount;
    use b3_wallet_lib::ledger::btc::network::BtcNetwork;
    use b3_wallet_lib::ledger::ckbtc::types::*;
    use b3_wallet_lib::ledger::types::*;
    use b3_wallet_lib::types::*;

    use ic_cdk::api::management_canister::bitcoin::{GetUtxosResponse, Satoshi, UtxoFilter};
    use ic_cdk::export::candid::export_service;

    #[test]
    fn generate_candid() {
        use std::io::Write;

        let mut file = std::fs::File::create("./b3_simple_wallet.did").unwrap();

        export_service!();

        let candid = __export_service();

        file.write_all(candid.as_bytes()).unwrap();

        assert!(true);
    }
}