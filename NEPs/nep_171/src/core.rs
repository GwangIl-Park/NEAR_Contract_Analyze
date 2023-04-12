use crate::token::{Token, TokenId};
use near_sdk::{AccountId, PromiseOrValue};
// 반환될 토큰의 기본 구조. 컨트랙트가 Approval Management, Metadata, 또는 다른
// 속성을 가진 extension이라면 이 구조체에 포함된다.

pub trait NonFungibleTokenCore {
    // 간단한 transfer. token_id의 토큰을 현재 소유자로부터 receiver_id에게 전송한다.
    //
    // Requirements
    // * 메서드 호출자는 보안을 위해 1yoctoⓃ를 첨부해야 한다.
    // * 토큰 소유자가 아닌 사람이 호출하면 panic이어야 하며, Approval Management를 사용하는 경우 approved된 account여야 한다.
    // * `approval_id` 는 Approval Management를 위한 것이다.
    // * Approval Management를 사용하는 경우, 컨트랙트는 전송 성공시 Approve된 계정을 무효화해야 한다.
    //
    // Arguments:
    // * `receiver_id`: 토큰을 받을 account
    // * `token_id`: 전송할 토큰
    // * `approval_id`: Approval Id. 2^53보다 작은 수이므로 JSON으로 표현 가능하다. Approval Management 참고
    // * `memo` (optional): indexing을 통해 활용 또는 transfer 정보 제공
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
    // token이 전송되면 true 리턴

    // 토큰을 전송하고 receiver컨트랙트에서 메서드를 호출한다.
    // 성공하면 NFT 컨트랙트의 nft_resolve_transfer 메서드에서 콜백에 대한 실행 결과가 반환된다.
    // native NEAR 토큰을 함수 호출에 attach하는 것과 비슷하다.
    // 이를 통해 NFT를 receiver 컨트랙트의 메서드 호출에 attach할 수 있다.
    //
    // Requirements:
    // * 메서드 호출자는 보안을 위해 1yoctoⓃ를 첨부해야 한다.
    // * 토큰 소유자가 아닌 사람이 호출하면 panic이어야 하며, Approval Management를 사용하는 경우 approved된 account여야 한다.
    // * receive 컨트랙트는 nft_on_transfer를 표준에 따라 구현해야 한다. 그렇지 않으면 nft_resolve_transfer는
    // * 실패하고 transfer가 롤백된다.
    // * 컨트랙트는 nft_resolve_transfer에 묘사된 행동을 구현해댜 한다.
    // * `approval_id` 는 Approval Management를 위한 것이다.
    // * Approval Management를 사용하는 경우, 컨트랙트는 전송 성공시 Approve된 계정을 무효화해야 한다.
    //
    // Arguments:
    // * `receiver_id`: 토큰을 받을 account
    // * `token_id`: 전송할 토큰
    // * `approval_id`: Approval Id. 2^53보다 작은 수이므로 JSON으로 표현 가능하다. Approval Management 참고
    // * `memo` (optional): indexing을 통해 활용 또는 transfer 정보 제공
    // * `msg`: receiving contract가 transfer를 처리하기 위해 필요한 정보
    // * 실행할 함수와 argument를 전달한다.
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool>;
    // token_id에 해당하는 token을 반환하거나 없다면 null을 반환한다.
    fn nft_token(&self, token_id: TokenId) -> Option<Token>;
}
