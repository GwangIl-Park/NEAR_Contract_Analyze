use crate::token::TokenId;
use near_sdk::{ext_contract, AccountId, PromiseOrValue};

//`nft_transfer_call`을 사용하여 NFT가 전송될 때 사용된다. receiver 컨트랙트에서 구현된다.
#[ext_contract(ext_nft_receiver)]
pub trait NonFungibleTokenReceiver {
    // NFT를 전송받은 후에 처리
    //
    // Requirements:
    // * 이 함수에 대한 호출은 화이트리스트에 포함된 NFT 집합으로 제한된다.
    //
    // Arguments:
    // * `sender_id`: `nft_transfer_call`의 sender
    // * `previous_owner_id`: NFT를 기존에 소유하고 있던 account, Approval Management를 사용하면
    // sender와 다를 수 있다.
    // * `token_id`: the `token_id` argument given to `nft_transfer_call`
    // * `msg`: 어떻게 처리될지 정보가 담김, 메서드 이름과 argument가 담김
    //
    // sender_id에게 토큰이 돌아가면 true가 떨어진다.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool>;
}
