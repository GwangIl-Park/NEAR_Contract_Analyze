use near_sdk::json_types::U128;
use near_sdk::AccountId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}

pub trait Payouts {
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: Option<u32>) -> Payout;
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: U128,
        max_len_payout: Option<u32>,
    ) -> Payout;
}
