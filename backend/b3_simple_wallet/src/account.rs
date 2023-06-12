use b3_helper_lib::{
    environment::Environment,
    identifier::AccountIdentifier,
    owner::caller_is_owner,
    revert,
    subaccount::Subaccount,
    tokens::Tokens,
    types::{AccountsNonce, BlockIndex, CanisterId, Cycles, Memo, NotifyTopUpResult},
};
use b3_wallet_lib::{
    account::WalletAccount,
    ledger::{
        btc::network::BtcNetwork,
        ckbtc::{
            minter::Minter,
            types::{BtcTxId, RetrieveBtcStatus, UtxoStatus},
        },
        types::{AddressMap, Balance, ChainEnum},
    },
    store::{
        with_account, with_account_mut, with_ledger, with_ledger_mut, with_wallet, with_wallet_mut,
    },
    types::{AccountId, WalletAccountView},
};
use ic_cdk::{
    api::management_canister::bitcoin::{GetUtxosResponse, Satoshi, UtxoFilter},
    export::candid::candid_method,
    query, update,
};
use std::str::FromStr;

// QUERY ---------------------------------------------------------------------

#[candid_method(query)]
#[query(guard = "caller_is_owner")]
pub fn get_account(account_id: AccountId) -> WalletAccount {
    with_account(&account_id, |account| account.clone()).unwrap_or_else(revert)
}

#[candid_method(query)]
#[query(guard = "caller_is_owner")]
pub fn get_account_count() -> usize {
    with_wallet(|s| s.accounts_len())
}

#[candid_method(query)]
#[query(guard = "caller_is_owner")]
pub fn get_account_counters() -> AccountsNonce {
    with_wallet(|s| s.counters().clone())
}

#[candid_method(query)]
#[query(guard = "caller_is_owner")]
pub fn get_account_views() -> Vec<WalletAccountView> {
    with_wallet(|s| s.account_views())
}

#[candid_method(query)]
#[query(guard = "caller_is_owner")]
pub fn get_account_view(account_id: AccountId) -> WalletAccountView {
    with_account(&account_id, |account| account.view()).unwrap_or_else(revert)
}

#[candid_method(query)]
#[query(guard = "caller_is_owner")]
pub fn get_addresses(account_id: AccountId) -> AddressMap {
    with_ledger(&account_id, |ledger| ledger.addresses().clone()).unwrap_or_else(revert)
}

#[candid_method(query)]
#[query(guard = "caller_is_owner")]
pub async fn retrieve_btc_status(
    network: BtcNetwork,
    block_index: BlockIndex,
) -> RetrieveBtcStatus {
    let minter = Minter(network);

    minter
        .retrieve_btc_status(block_index)
        .await
        .unwrap_or_else(revert)
}

// UPDATE ---------------------------------------------------------------------

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub fn account_create(env: Option<Environment>, name: Option<String>) {
    let subaccount = with_wallet(|s| s.new_subaccount(env));

    let new_account = WalletAccount::from(subaccount);

    with_wallet_mut(|s| s.insert_account(new_account, name));
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub fn account_rename(account_id: AccountId, name: String) {
    with_account_mut(&account_id, |a| a.rename(name)).unwrap_or_else(revert)
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub fn account_hide(account_id: AccountId) {
    with_account_mut(&account_id, |a| a.hide()).unwrap_or_else(revert)
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub fn account_remove(account_id: AccountId) {
    with_wallet_mut(|s| s.remove_account(&account_id)).unwrap_or_else(revert);
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub fn account_remove_address(account_id: AccountId, chain: ChainEnum) {
    with_ledger_mut(&account_id, |ledger| ledger.remove_address(chain))
        .unwrap_or_else(revert)
        .unwrap_or_else(revert);
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub fn account_restore(env: Environment, nonce: u64) {
    let subaccount = Subaccount::new(env, nonce);

    with_wallet_mut(|s| s.restore_account(subaccount)).unwrap_or_else(revert);
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_balance(account_id: AccountId, chain: ChainEnum) -> Balance {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let balance = ledger.balance(chain).await;

    match balance {
        Ok(balance) => balance,
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_icp_balance(account_id: AccountId) -> Balance {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let balance = ledger.balance(ChainEnum::ICP).await;

    match balance {
        Ok(balance) => balance,
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_ckbtc_balance(account_id: AccountId, network: BtcNetwork) -> Balance {
    let ledger = with_ledger_mut(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let balance = ledger.balance(ChainEnum::CKBTC(network)).await;

    match balance {
        Ok(balance) => balance,
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_icrc_balance(account_id: AccountId, canister_id: CanisterId) -> Balance {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let balance = ledger.balance(ChainEnum::ICRC(canister_id)).await;

    match balance {
        Ok(balance) => balance,
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_send_icp(
    account_id: AccountId,
    to: String,
    amount: Tokens,
    fee: Option<Tokens>,
    memo: Option<Memo>,
) -> BlockIndex {
    let to = AccountIdentifier::from_str(&to).unwrap_or_else(revert);

    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let result = ledger
        .transfer(to, amount, fee, memo)
        .await
        .unwrap_or_else(revert);

    match result {
        Ok(result) => result,
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_send(account_id: AccountId, chain: ChainEnum, to: String, amount: u64) {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    ledger.send(chain, to, amount).await.unwrap_or_else(revert);
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_btc_utxos(
    account_id: AccountId,
    network: BtcNetwork,
    filter: Option<UtxoFilter>,
) -> GetUtxosResponse {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let utxos = ledger.bitcoin_get_utxos(network.into(), filter).await;

    match utxos {
        Ok(utxos) => utxos,
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_btc_fees(network: BtcNetwork, num_blocks: u8) -> u64 {
    let rate = network.fee_rate(num_blocks).await;

    match rate {
        Ok(rate) => rate,
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_balance_btc(
    account_id: AccountId,
    network: BtcNetwork,
    min_confirmations: Option<u32>,
) -> Balance {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let balance = ledger
        .bitcoin_balance(network.into(), min_confirmations)
        .await;

    match balance {
        Ok(balance) => balance,
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_update_balance(account_id: AccountId, network: BtcNetwork) -> Vec<UtxoStatus> {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let ckbtc = ledger
        .ckbtc(network)
        .ok_or(format!("Ckbtc not initialized!"))
        .unwrap_or_else(revert);

    let result = ckbtc.update_balance().await.unwrap_or_else(revert);

    match result {
        Ok(result) => {
            // TODO: this need to work with utxos that are in the pending state
            with_ledger_mut(&account_id, |ledger| {
                // we already checked that ckbtc is initialized
                let ckbtc = ledger.ckbtc_mut(network).unwrap();

                ckbtc.remove_pending();
            })
            .unwrap_or_else(revert);

            result
        }
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_swap_btc_to_ckbtc(
    account_id: AccountId,
    network: BtcNetwork,
    amount: Satoshi,
) -> BtcTxId {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let result = ledger.swap_btc_to_ckbtc(network, amount).await;

    match result {
        Ok(tx_id) => {
            with_ledger_mut(&account_id, |ledger| {
                // we already checked that ckbtc is initialized
                let ckbtc = ledger.ckbtc_mut(network).unwrap();

                ckbtc.add_pending(tx_id.to_string());
            })
            .unwrap_or_else(revert);

            tx_id.to_string()
        }
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_swap_ckbtc_to_btc(
    account_id: AccountId,
    network: BtcNetwork,
    btc_address: String,
    amount: Satoshi,
) -> BtcTxId {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let ckbtc = ledger
        .ckbtc(network)
        .ok_or(format!("Ckbtc not initialized!"))
        .unwrap_or_else(revert);

    let result = ckbtc.swap_ckbtc_to_btc(btc_address, amount).await;

    match result {
        Ok(result) => result.block_index.to_string(),
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_send_btc(
    account_id: AccountId,
    network: BtcNetwork,
    to: String,
    amount: Satoshi,
) -> BtcTxId {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let result = ledger.bitcoin_transfer(network, &to, amount).await;

    match result {
        Ok(tx_id) => tx_id.to_string(),
        Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_top_up_and_notify(
    account_id: AccountId,
    amount: Tokens,
    canister_id: Option<CanisterId>,
    fee: Option<Tokens>,
) -> Cycles {
    let ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let canister_id = canister_id.unwrap_or(ic_cdk::id());

    let result = ledger
        .topup_and_notify_top_up(canister_id, amount, fee)
        .await
        .unwrap_or_else(revert);

    match result {
        NotifyTopUpResult::Ok(result) => result,
        NotifyTopUpResult::Err(err) => revert(err),
    }
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub async fn account_create_address(account_id: AccountId, chain_type: ChainEnum) {
    let mut ledger = with_ledger(&account_id, |ledger| ledger.clone()).unwrap_or_else(revert);

    let ecdsa = match chain_type {
        ChainEnum::BTC(_) | ChainEnum::EVM(_) => {
            if !ledger.is_public_key_set() {
                let ecdsa = ledger.ecdsa_public_key().await.unwrap_or_else(revert);

                ledger
                    .set_ecdsa_public_key(ecdsa.clone())
                    .unwrap_or_else(revert);

                Some(ecdsa)
            } else {
                None
            }
        }
        _ => None,
    };

    let chain = ledger
        .new_chain(chain_type.clone())
        .await
        .unwrap_or_else(revert);

    with_ledger_mut(&account_id, |ledger| {
        if let Some(ecdsa) = ecdsa {
            ledger.set_ecdsa_public_key(ecdsa).unwrap_or_else(revert);
        }

        ledger.insert_chain(chain_type, chain)
    })
    .unwrap_or_else(revert);
}

#[candid_method(update)]
#[update(guard = "caller_is_owner")]
pub fn reset_wallet() {
    with_wallet_mut(|s| s.reset());
}