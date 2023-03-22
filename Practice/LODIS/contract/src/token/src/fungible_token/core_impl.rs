use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{
    assert_one_yocto, env, log, require, AccountId, Balance, Gas, IntoStorageKey, PromiseOrValue,
    PromiseResult, StorageUsage,
};

use crate::fungible_token::core::FungibleTokenCore;
use crate::fungible_token::events::{FtBurn, FtTransfer};
use crate::fungible_token::receiver::ext_ft_receiver;
use crate::fungible_token::resolver::{ext_ft_resolver, FungibleTokenResolver};

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);

const ERR_TOTAL_SUPPLY_OVERFLOW: &str = "Total supply overflow";

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    pub balance_map: LookupMap<AccountId, Balance>,
    pub total_supply: Balance,
    pub account_storage_usage: StorageUsage,
}

impl FungibleToken {
    //FungibleToken 구조체 초기화
    pub fn new<S>(prefix: S) -> Self
    where
        S: IntoStorageKey,
    {
        let mut this = Self {
            balance_map: LookupMap::new(prefix),
            total_supply: 0,
            account_storage_usage: 0,
        };
        this.mesure_account_storage_usage();
        this
    }
    //account당 storage 사용량 계산
    pub fn mesure_account_storage_usage(&mut self) {
        let initial_usage = env::storage_usage();
        let test_account = AccountId::new_unchecked("a".repeat(64));
        self.balance_map.insert(&test_account, &0u128);
        self.account_storage_usage = env::storage_usage() - initial_usage;
        self.balance_map.remove(&test_account);
    }
    //account의 balance 추출
    pub fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        match self.balance_map.get(&account_id) {
            Some(balance) => balance,
            None => env::panic_str(format!("Account {} is not registered", &account_id).as_str()),
        }
    }
    //입금
    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_add(amount) {
            self.balance_map.insert(&account_id, &new_balance);
            self.total_supply = self
                .total_supply
                .checked_add(amount)
                .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW))
        } else {
            env::panic_str("Balance Overflow")
        }
    }
    //출금
    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id);
        if let Some(new_balance) = balance.checked_sub(amount) {
            self.balance_map.insert(account_id, &new_balance);
            self.total_supply = self
                .total_supply
                .checked_sub(amount)
                .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW))
        } else {
            env::panic_str("Not Enough Balance");
        }
    }
    //transfer
    pub fn internal_transfer(
        &mut self,
        sender: &AccountId,
        receiver: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        require!(sender != receiver, "Sender, Receiver should be different");
        require!(amount > 0, "Amount should be larger than 0");
        self.internal_withdraw(sender, amount);
        self.internal_deposit(receiver, amount);
    }
    //account 등록
    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        if self.balance_map.insert(account_id, &0).is_some() {
            env::panic_str("Already Exists")
        }
    }
}

impl FungibleTokenCore for FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        self.internal_transfer(
            &env::predecessor_account_id(),
            &receiver_id,
            amount.into(),
            memo,
        );
    }

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<String> {
        assert_one_yocto();
        let sender = env::predecessor_account_id();
        self.internal_transfer(&sender, &receiver_id, amount.into(), memo);

        let receiver_gas = env::prepaid_gas()
            .0
            .checked_sub(GAS_FOR_FT_TRANSFER_CALL.0)
            .unwrap_or_else(|| env::panic_str("prepaid gas overflow"));

        ext_ft_receiver::ext(receiver_id)
            .with_static_gas(receiver_gas.into())
            .ft_on_transfer(sender, amount, msg)
            .into()
    }

    fn ft_total_supply(&self) -> U128 {
        self.total_supply.into()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.balance_map.get(&account_id).unwrap_or(0).into()
    }
}

impl FungibleToken {
    /// Internal method that returns the amount of burned tokens in a corner case when the sender
    /// has deleted (unregistered) their account while the `ft_transfer_call` was still in flight.
    /// Returns (Used token amount, Burned token amount)
    pub fn internal_ft_resolve_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> (u128, u128) {
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    amount
                }
            }
            PromiseResult::Failed => amount,
        };

        if unused_amount > 0 {
            let receiver_balance = self.balance_map.get(&receiver_id).unwrap_or(0);
            if receiver_balance > 0 {
                let refund_amount = std::cmp::min(receiver_balance, unused_amount);
                if let Some(new_receiver_balance) = receiver_balance.checked_sub(refund_amount) {
                    self.balance_map.insert(&receiver_id, &new_receiver_balance);
                } else {
                    env::panic_str("The receiver account doesn't have enough balance");
                }

                if let Some(sender_balance) = self.balance_map.get(sender_id) {
                    if let Some(new_sender_balance) = sender_balance.checked_add(refund_amount) {
                        self.balance_map.insert(sender_id, &new_sender_balance);
                    } else {
                        env::panic_str("Sender balance overflow");
                    }

                    FtTransfer {
                        old_owner_id: &receiver_id,
                        new_owner_id: sender_id,
                        amount: &U128(refund_amount),
                        memo: Some("refund"),
                    }
                    .emit();
                    let used_amount = amount
                        .checked_sub(refund_amount)
                        .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW));
                    return (used_amount, 0);
                } else {
                    // Sender's account was deleted, so we need to burn tokens.
                    self.total_supply = self
                        .total_supply
                        .checked_sub(refund_amount)
                        .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW));
                    log!("The account of the sender was deleted");
                    FtBurn {
                        owner_id: &receiver_id,
                        amount: &U128(refund_amount),
                        memo: Some("refund"),
                    }
                    .emit();
                    return (amount, refund_amount);
                }
            }
        }
        (amount, 0)
    }
}

impl FungibleTokenResolver for FungibleToken {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        self.internal_ft_resolve_transfer(&sender_id, receiver_id, amount)
            .0
            .into()
    }
}
