use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use nep_177::TokenMetadata;
use std::collections::HashMap;

//NFT의 토큰 ID는 문자열로 표현해야 한다. 체인에 구애받지 않는 표준에 따라 ID를 사용하기 위해, Bridge를 고려할 때, 유연성을 확보할 수 있다.
pub type TokenId = String;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema))]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: Option<TokenMetadata>,
    pub approved_account_ids: Option<HashMap<AccountId, u64>>,
    pub payout: Option<HashMap<AccountId, U128>>,
}
