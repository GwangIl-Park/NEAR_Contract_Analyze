use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, Balance};

mod deploy;
mod manager;

const NEAR_PER_STORAGE: Balance = 10_000_000_000_000_000_000; // 10e19yⓃ
const DEFAULT_CONTRACT: &[u8] = include_bytes!("./contract/hello_near.wasm");

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    code: Vec<u8>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            code: DEFAULT_CONTRACT.to_vec(),
        }
    }
}
