// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::ext_contract;
use near_sdk::{env, near_bindgen, AccountId};

#[ext_contract(ext_ft_receiver)]

pub trait Whitelist {
    fn signup_user(&mut self, user_id: String, user_address: AccountId) -> bool;
    fn get_address(&self, user_id: String) -> AccountId;
    fn is_user_by_id(&self, user_id: String) -> bool;
    fn is_user_by_address(&self, user_address: AccountId) -> bool;
    fn is_valid_user_by_id(&self, user_id: String, user_address: AccountId) -> bool;
    fn is_valid_user_by_address(&self, user_id: String, user_address: AccountId) -> bool;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct WhitelistStr {
    user_data_by_address: LookupMap<AccountId, String>,
    user_data_by_id: LookupMap<String, AccountId>,
}

impl Default for WhitelistStr {
    fn default() -> Self {
        Self {
            user_data_by_address: LookupMap::new(b"a"),
            user_data_by_id: LookupMap::new(b"i"),
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Whitelist for WhitelistStr {
    fn signup_user(&mut self, user_id: String, user_address: AccountId) -> bool {
        assert!(
            user_address == env::predecessor_account_id(),
            "signup others"
        );
        assert!(
            !self.is_user_by_address(user_address.clone()),
            "registered user"
        );
        assert!(!self.is_user_by_id(user_id.clone()), "registered user");
        self.user_data_by_address.insert(&user_address, &user_id);
        self.user_data_by_id.insert(&user_id, &user_address);
        true
    }

    fn get_address(&self, user_id: String) -> AccountId {
        AccountId::from(
            self.user_data_by_id
                .get(&user_id)
                .unwrap_or_else(|| env::panic_str("address not found")),
        )
    }

    fn is_user_by_id(&self, user_id: String) -> bool {
        !self.user_data_by_id.get(&user_id).is_none()
    }

    fn is_user_by_address(&self, user_address: AccountId) -> bool {
        !self.user_data_by_address.get(&user_address).is_none()
    }

    fn is_valid_user_by_id(&self, user_id: String, user_address: AccountId) -> bool {
        let address = self
            .user_data_by_id
            .get(&user_id)
            .unwrap_or_else(|| env::panic_str("address not found"));
        address == user_address
    }

    fn is_valid_user_by_address(&self, user_id: String, user_address: AccountId) -> bool {
        let id = self
            .user_data_by_address
            .get(&user_address)
            .unwrap_or_else(|| env::panic_str("address not found"));
        id == user_id
    }
}
