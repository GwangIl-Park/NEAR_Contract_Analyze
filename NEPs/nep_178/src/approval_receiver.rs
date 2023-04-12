use near_sdk::{ext_contract, AccountId};
use nep_171::token::TokenId;

//NFT 컨트랙트가 account에 대한 approve를 추가할 때 호출됨
#[ext_contract(ext_nft_approval_receiver)]
pub trait NonFungibleTokenApprovalReceiver {
    ///
    /// Notes
    ///
    /// * 컨트랙트는 `predecessor_account_id`로부터 토큰 컨트랙트 ID를 안다.
    ///
    /// Arguments:
    /// * `token_id`: 토큰 ID
    /// * `owner_id`: 토큰의 owner
    /// * `approval_id`: approve에 대해 NFT 컨트랙트가 저장한 ID
    ///   2^53으로 제한
    /// * `msg`: argument와 method 이름
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) -> near_sdk::PromiseOrValue<String>; // TODO: how to make "any"?
}
