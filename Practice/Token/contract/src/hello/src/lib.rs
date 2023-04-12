use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PromiseOrValue};
pub use token::fungible_token::receiver::FungibleTokenReceiver;

// Define the default message
const DEFAULT_MESSAGE: &str = "Default";

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Hello {
    message: String,
    balance_map: LookupMap<AccountId, Balance>,
}

impl Default for Hello {
    fn default() -> Self {
        Self {
            message: DEFAULT_MESSAGE.to_string(),
            balance_map: LookupMap::new(b"m"),
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Hello {
    pub fn get_message(&self) -> String {
        self.message.clone()
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message
    }
    pub fn donate_token(&mut self, sender_id: AccountId, amount: Balance) {
        let balance = self.balance_of(&sender_id);
        self.balance_map.insert(&sender_id, &(balance + amount));
    }
    pub fn balance_of(&self, sender_id: &AccountId) -> Balance {
        self.balance_map.get(&sender_id).unwrap_or(0)
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Hello {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let split_messages: Vec<&str> = msg.split(",").collect(); //ex) set_message,Hello
        log!(
            "in {} tokens from @{} ft_on_transfer, msg = {}",
            amount.0,
            sender_id.as_ref(),
            msg
        );
        match split_messages[0] {
            "set_message" => {
                Self::ext(env::current_account_id()).set_message(split_messages[1].to_string());
                near_sdk::PromiseOrValue::Value(amount.into())
            }
            "donate" => {
                Self::ext(env::current_account_id()).donate_token(sender_id, amount.into());

                near_sdk::PromiseOrValue::Value(U128::from(0))
            }
            _ => env::panic_str("Unnown"),
        }
    }
}
