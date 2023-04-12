use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::{env, log, near_bindgen, AccountId, Balance};
use std::thread;
use std::time::Duration;

const DEFAULT_MESSAGE: &str = "Hello";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    message: String,
}

impl Default for Contract {
    fn default() -> Self {
        log!("default called");
        Self {
            message: DEFAULT_MESSAGE.to_string(),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_greeting(&self) -> String {
        return self.message.clone();
    }

    pub fn set_greeting(&mut self, message: String) {
        log!("Saving greeting {}", message);
        self.message = message;
    }
}
