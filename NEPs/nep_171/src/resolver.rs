use crate::token::TokenId;
use near_sdk::{ext_contract, AccountId};
use std::collections::HashMap;

//`nft_transfer_call`을 사용하여 NFT가 전송될 때 사용된다. NFT 컨트랙트에서 구현된다.
#[ext_contract(ext_nft_resolver)]
pub trait NonFungibleTokenResolver {
    // nft_transfer_call을 마무리한다.
    //
    // The `nft_transfer_call` process:
    //
    // 1. sender가 NFT 컨트랙트에서 nft_transfer_call을 호출한다.
    // 2. 토큰이 sender에서 receiver로 전달된다.
    // 3. receiver 컨트랙트에서 nft_on_transfer가 호출된다.
    // 4+. receiver 컨트랙트는 다른 cross-contract call을 할 것이다.
    // N. nft_resolve_transfer로 promise를 resolve한다.
    //
    // Requirements:
    // * 컨트랙트 자신만이 이 함수를 호출해야 한다.
    // * promise가 실패하면 token 전송을 롤백해야 한다.
    // * promise가 true로 떨어지면 토큰을 owner_id에게 돌려줘야 한다.
    //
    // Arguments:
    // * `owner_id`: NFT의 원래 주인
    // * `receiver_id`: 토큰을 받을 account
    // * `token_id`: 전송할 토큰
    // * `approved_account_ids`: Approval Management를 사용한다면, 컨트랙트는 기존 approved account
    // 를 이 argument로 제공해야 한다. 그리고 승인된 계정돠 승인 ID를 복원해야 한다.
    //
    // 토큰이 receiver_id로 전송된다면 true를 리턴한다.
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approvals: Option<HashMap<AccountId, u64>>,
    ) -> bool;
}
