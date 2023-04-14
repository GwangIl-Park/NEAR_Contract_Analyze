use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PromiseOrValue};
pub use nep_171::receiver::NonFungibleTokenReceiver;
pub use nep_171::token::TokenId;
pub use nep_178::approval_receiver::NonFungibleTokenApprovalReceiver;
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

#[near_bindgen]
impl NonFungibleTokenReceiver for Hello {
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        let split_messages: Vec<&str> = msg.split(",").collect(); //ex) set_message,Hello
        match split_messages[0] {
            "set_message" => {
                Self::ext(env::current_account_id()).set_message(split_messages[1].to_string());
                PromiseOrValue::Value(false)
            }
            _ => PromiseOrValue::Value(true),
        }
    }
}

#[near_bindgen]
impl NonFungibleTokenApprovalReceiver for Hello {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) -> PromiseOrValue<String> {
        let split_messages: Vec<&str> = msg.split(",").collect(); //ex) set_message,Hello
        match split_messages[0] {
            "set_message" => {
                Self::ext(env::current_account_id()).set_message(split_messages[1].to_string());
                PromiseOrValue::Value("success".to_string())
            }
            _ => PromiseOrValue::Value("fail".to_string()),
        }
    }
}
