use crate::core_impl::FungibleToken;
use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, env, log, AccountId, Balance, Promise};
use nep_145::{StorageBalance, StorageBalanceBounds, StorageManagement};

impl FungibleToken {
    //account 등록을 삭제한다
    pub fn internal_storage_unregister(
        &mut self,
        force: Option<bool>,
    ) -> Option<(AccountId, Balance)> {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let force = force.unwrap_or(false); //balance가 0이 아니어도 삭제할 수 있도록 하는 옵션
        if let Some(balance) = self.balance_map.get(&account_id) {
            if balance == 0 || force {
                self.balance_map.remove(&account_id);
                self.total_supply -= balance;
                Promise::new(account_id.clone()).transfer(self.storage_balance_bounds().min.0 + 1); //account가 사용하고 있던 storage 비용을 돌려줌
                Some((account_id, balance))
            } else {
                env::panic_str(
                    "Can't unregister the account with the positive balance without force",
                )
            }
        } else {
            log!("account {} is not registered", &account_id);
            None
        }
    }

    //스토리지 사용량 반환
    fn internal_storage_balance_of(&self, account_id: &AccountId) -> Option<StorageBalance> {
        if self.balance_map.contains_key(account_id) {
            Some(StorageBalance {
                total: self.storage_balance_bounds().min,
                available: 0.into(), //사용가능한 storage양 같은데 이 예제에서는 어차피 사용량이 고정이기 때문에 쓰지 않는 듯
            })
        } else {
            None
        }
    }
}

impl StorageManagement for FungibleToken {
    //storage 비용을 입금하고 사용자등록을 한다.
    #[allow(unused_variables)] //선언되었지만 사용하지 않는 변수 허용 (registration_only)
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        if self.balance_map.contains_key(&account_id) {
            log!("account already registered");
            if amount > 0 {
                Promise::new(env::predecessor_account_id()).transfer(amount);
            }
        } else {
            let min_balance = self.storage_balance_bounds().min.0;
            if amount < min_balance {
                env::panic_str("attached deposit is less than min")
            }
            self.internal_register_account(&account_id);
            let refund = amount - min_balance;
            if refund > 0 {
                Promise::new(env::predecessor_account_id()).transfer(refund);
            }
        }
        self.internal_storage_balance_of(&account_id).unwrap()
    }
    //사용가능한 storage를 확인한다. (amount가 있다면 인출하는데 이 예제에서는 어차피 사용량이 고정이기 때문에 쓰지 않는 듯)
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        assert_one_yocto();
        let predecessor_account_id = env::predecessor_account_id();
        if let Some(storage_balance) = self.internal_storage_balance_of(&predecessor_account_id) {
            match amount {
                Some(amount) if amount.0 > 0 => {
                    env::panic_str("The amount is greater than storage balance")
                }
                _ => storage_balance,
            }
        } else {
            env::panic_str(
                format!("account {} is not registered", &predecessor_account_id).as_str(),
            )
        }
    }

    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.internal_storage_unregister(force).is_some()
    }

    //account당 storage 가격 범위
    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(self.account_storage_usage) * env::storage_byte_cost(); //account당 storage사용량 * byte당 가격
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()), //값이 있다면 max에 담기는 것인데 이 예제에서는 어차피 사용량이 고정이기 때문에 쓰지 않는 듯
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.internal_storage_balance_of(&account_id)
    }
}
