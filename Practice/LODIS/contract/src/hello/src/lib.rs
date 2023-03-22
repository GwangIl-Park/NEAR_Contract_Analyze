use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, PromiseOrValue};

pub use token::fungible_token::receiver::FungibleTokenReceiver;

// Define the default message
const DEFAULT_MESSAGE: &str = "Default";

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Hello {
    message: String,
}

impl Default for Hello {
    fn default() -> Self {
        Self {
            message: DEFAULT_MESSAGE.to_string(),
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
}

impl FungibleTokenReceiver for Hello {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<String> {
        let parsed_message = &msg[0..3];
        log!(parsed_message);
        match parsed_message {
            "get" => near_sdk::PromiseOrValue::Value(self.get_message()),
            "set" => {
                let new_message = &msg[11..];
                near_sdk::PromiseOrValue::Promise(
                    Self::ext(env::current_account_id()).set_message(new_message.to_string()),
                )
            }
            _ => env::panic_str("Unknown Message"),
        }
    }
}
