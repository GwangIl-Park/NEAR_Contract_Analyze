use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, StorageUsage};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct NonFungibleToken {
    pub owner_id: AccountId,
    pub extra_storage_in_bytes_per_token: StorageUsage,
    pub owner_by_id: TreeMap<TokenId, AccountId>,
}
