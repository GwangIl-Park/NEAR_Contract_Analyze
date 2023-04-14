use crate::core_impl::NonFungibleToken;
use near_sdk::env::panic_str;
use near_sdk::json_types::U128;
use near_sdk::{require, AccountId};
use nep_171::core::NonFungibleTokenCore;
use nep_199::payout::Payout;
use nep_199::payout::Payouts;
use std::collections::HashMap;

impl Payouts for NonFungibleToken {
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: Option<u32>) -> Payout {
        let mut return_payout = Payout {
            payout: HashMap::new(),
        };
        let royalty;
        if let Some(royalty_by_id) = &self.royalty_by_id {
            royalty = royalty_by_id;
        } else {
            panic_str("no royalty");
        }

        let payout = royalty.get(&token_id).unwrap_or(HashMap::new());

        if let Some(max_len_payout) = max_len_payout {
            if payout.len() > max_len_payout.try_into().unwrap() {
                panic_str("payout length overflow");
            }
        }

        let mut check_balance: u128 = 0;
        for (k, v) in payout.iter() {
            check_balance += v.0;
            return_payout.payout.insert(k.clone(), *v);
        }
        require!(
            balance >= U128(check_balance),
            "balance should be larger than payout"
        );
        return_payout
    }
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: U128,
        max_len_payout: Option<u32>,
    ) -> Payout {
        let payout = self.nft_payout(token_id.clone(), balance, max_len_payout);
        self.nft_transfer(receiver_id, token_id.clone(), approval_id, memo);
        payout
    }
}
