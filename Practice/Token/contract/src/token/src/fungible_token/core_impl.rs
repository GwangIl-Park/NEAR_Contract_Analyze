use crate::fungible_token::core::FungibleTokenCore;
use crate::fungible_token::events::{FtBurn, FtTransfer};
use crate::fungible_token::receiver::ext_ft_receiver;
use crate::fungible_token::resolver::{ext_ft_resolver, FungibleTokenResolver};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{
    assert_one_yocto, env, log, require, AccountId, Balance, Gas, IntoStorageKey, PromiseOrValue,
    PromiseResult, StorageUsage,
};

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0); //GAS_FOR_RESOLVE_TRANSFER의 타입이 Gas기 때문에 일반적인 정수형 타입과 계산이 불가, .0을 붙이면 타입이 가진 값을 가지고 옴

const ERR_TOTAL_SUPPLY_OVERFLOW: &str = "Total supply overflow";

#[derive(BorshDeserialize, BorshSerialize)]
pub struct FungibleToken {
    pub balance_map: LookupMap<AccountId, Balance>,
    pub total_supply: Balance,
    pub account_storage_usage: StorageUsage,
}

//internal 함수들이라고 보면됨
impl FungibleToken {
    //FungibleToken 구조체 초기화
    pub fn new<S>(prefix: S) -> Self
    where
        //제네릭 타입 매개변수의 제약 조건을 지정
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
            //switch문과 비슷
            Some(balance) => balance,
            None => env::panic_str(format!("Account {} is not registered", &account_id).as_str()),
        }
    }

    //입금
    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        let balance = self.internal_unwrap_balance_of(account_id); //이 과정 때문에 먼저 등록하는 과정이 필요함
        if let Some(new_balance) = balance.checked_add(amount) {
            //checked_add의 return값이 Some인 경우에 실행되며 그 값이 new_balance에 담김
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
        FtTransfer {
            old_owner_id: sender,
            new_owner_id: receiver,
            amount: &U128(amount),
            memo: memo.as_deref(),
        }
        .emit();
    }

    //account 등록
    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        if self.balance_map.insert(account_id, &0).is_some() {
            //이미 있는 key로 추가하면 value가 떨어지고, 아니면 None이 떨어짐
            env::panic_str("Already Exists")
        }
    }
}

impl FungibleTokenCore for FungibleToken {
    //외부 호출용 transfer
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto(); //1yoctoNEAR를 deposit하지 않으면 assert (중요한 기능을 할때, 호출자가 full-access key를 통해 호출하도록 강제하기 위해 사용)
        self.internal_transfer(
            &env::predecessor_account_id(),
            &receiver_id,
            amount.into(), //타입을 맞춰서 변환
            memo,
        );
    }

    //transfer이후 외부 contract 함수 호출
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
        let amount: Balance = amount.into();
        let receiver_gas = env::prepaid_gas()
            .0
            .checked_sub(GAS_FOR_FT_TRANSFER_CALL.0)
            .unwrap_or_else(|| env::panic_str("prepaid gas overflow")); //지불된 가스에서 GAS_FOR_FT_TRANSFER_CALL 를 뺀 값

        ext_ft_receiver::ext(receiver_id.clone())
            .with_static_gas(receiver_gas.into())
            .ft_on_transfer(sender.clone(), amount.into(), msg)
            .then(
                ext_ft_resolver::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .ft_resolve_transfer(sender, receiver_id, amount.into()),
            )
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
    //ft_transfer_call의 후처리를 위함
    pub fn internal_ft_resolve_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> (u128, u128) {
        let amount: Balance = amount.into();

        // Get the unused amount from the `ft_on_transfer` call result.
        let unused_amount = match env::promise_result(0) {
            //ft_on_transfer의 결과
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                //value에는 ft_on_transfer의 결과가 담김
                if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                    log!("AAA{}", unused_amount.0);
                    std::cmp::min(amount, unused_amount.0)
                } else {
                    log!("BBB");
                    amount
                }
            }
            PromiseResult::Failed => amount,
        };

        if unused_amount > 0 {
            let receiver_balance = self.balance_map.get(&receiver_id).unwrap_or(0);
            if receiver_balance > 0 {
                let refund_amount = std::cmp::min(receiver_balance, unused_amount); //receiver balance 처리
                if let Some(new_receiver_balance) = receiver_balance.checked_sub(refund_amount) {
                    self.balance_map.insert(&receiver_id, &new_receiver_balance);
                } else {
                    env::panic_str("The receiver account doesn't have enough balance");
                }

                if let Some(sender_balance) = self.balance_map.get(sender_id) {
                    if let Some(new_sender_balance) = sender_balance.checked_add(refund_amount) {
                        //sender balance 처리
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
                    //sender가 삭제된 경우
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
            .0 //앞에있는 값만 담김
            .into()
    }
}
