use crate::core_impl::NonFungibleToken;
use near_sdk::{assert_one_yocto, env, require, AccountId, Gas, Promise};
use nep_171::token::TokenId;
use nep_171::utils::{
    assert_at_least_one_yocto, bytes_for_approved_account_id, refund_approved_account_ids,
    refund_approved_account_ids_iter, refund_deposit,
};
/// Common implementation of the [approval management standard](https://nomicon.io/Standards/NonFungibleToken/ApprovalManagement.html) for NFTs.
/// on the contract/account that has just been approved. This is not required to implement.
use nep_178::approval::NonFungibleTokenApproval;
use nep_178::approval_receiver::ext_nft_approval_receiver;

const GAS_FOR_NFT_APPROVE: Gas = Gas(10_000_000_000_000);

fn expect_token_found<T>(option: Option<T>) -> T {
    option.unwrap_or_else(|| env::panic_str("Token not found"))
}

fn expect_approval<T>(option: Option<T>) -> T {
    option.unwrap_or_else(|| env::panic_str("next_approval_by_id must be set for approval ext"))
}

impl NonFungibleTokenApproval for NonFungibleToken {
    fn nft_approve(
        &mut self,
        token_id: TokenId,
        account_id: AccountId,
        msg: Option<String>,
    ) -> Option<Promise> {
        assert_at_least_one_yocto();

        //NFT가 approval을 지원하는지 체크
        let approvals_by_id = self
            .approvals_by_id
            .as_mut()
            .unwrap_or_else(|| env::panic_str("NFT does not support Approval Management"));

        //tokenId로 owner_id 가져옴
        let owner_id = expect_token_found(self.owner_by_id.get(&token_id));

        require!(
            env::predecessor_account_id() == owner_id,
            "Predecessor must be token owner."
        );

        //next_approval_by_id필드가 있는지 체크
        let next_approval_id_by_id = expect_approval(self.next_approval_id_by_id.as_mut());
        // token_id에 대한 approval맵 가져옴
        let approved_account_ids = &mut approvals_by_id.get(&token_id).unwrap_or_default();
        //token_id에 대한 next_approval_id 가져옴
        let approval_id: u64 = next_approval_id_by_id.get(&token_id).unwrap_or(1u64);
        // account에 next_approval_id를 넣음 (기존 값 리턴)
        let old_approval_id = approved_account_ids.insert(account_id.clone(), approval_id);

        // token_id에 대한 approve맵을 업데이트
        approvals_by_id.insert(&token_id, approved_account_ids);

        // 토큰에 대한 next_approval_id +1
        next_approval_id_by_id.insert(&token_id, &(approval_id + 1));

        // 새로운 account가 등록된 경우 추가
        let storage_used = if old_approval_id.is_none() {
            bytes_for_approved_account_id(&account_id)
        } else {
            0
        };
        refund_deposit(storage_used);

        // msg가 있으면 nft_on_approve 실행
        msg.map(|msg| {
            ext_nft_approval_receiver::ext(account_id)
                .with_static_gas(env::prepaid_gas() - GAS_FOR_NFT_APPROVE)
                .nft_on_approve(token_id, owner_id, approval_id, msg)
        })
    }

    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        assert_one_yocto();
        let approvals_by_id = self.approvals_by_id.as_mut().unwrap_or_else(|| {
            env::panic_str("NFT does not support Approval Management");
        });

        let owner_id = expect_token_found(self.owner_by_id.get(&token_id));
        let predecessor_account_id = env::predecessor_account_id();

        require!(
            predecessor_account_id == owner_id,
            "Predecessor must be token owner."
        );

        // if token has no approvals, do nothing
        if let Some(approved_account_ids) = &mut approvals_by_id.get(&token_id) {
            // if account_id was already not approved, do nothing
            if approved_account_ids.remove(&account_id).is_some() {
                refund_approved_account_ids_iter(
                    predecessor_account_id,
                    core::iter::once(&account_id),
                );
                // if this was the last approval, remove the whole HashMap to save space.
                if approved_account_ids.is_empty() {
                    approvals_by_id.remove(&token_id);
                } else {
                    // otherwise, update approvals_by_id with updated HashMap
                    approvals_by_id.insert(&token_id, approved_account_ids);
                }
            }
        }
    }

    fn nft_revoke_all(&mut self, token_id: TokenId) {
        assert_one_yocto();
        let approvals_by_id = self.approvals_by_id.as_mut().unwrap_or_else(|| {
            env::panic_str("NFT does not support Approval Management");
        });

        let owner_id = expect_token_found(self.owner_by_id.get(&token_id));
        let predecessor_account_id = env::predecessor_account_id();

        require!(
            predecessor_account_id == owner_id,
            "Predecessor must be token owner."
        );

        // if token has no approvals, do nothing
        if let Some(approved_account_ids) = &mut approvals_by_id.get(&token_id) {
            // otherwise, refund owner for storage costs of all approvals...
            refund_approved_account_ids(predecessor_account_id, approved_account_ids);
            // ...and remove whole HashMap of approvals
            approvals_by_id.remove(&token_id);
        }
    }

    fn nft_is_approved(
        &self,
        token_id: TokenId,
        approved_account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        expect_token_found(self.owner_by_id.get(&token_id));

        let approvals_by_id = if let Some(a) = self.approvals_by_id.as_ref() {
            a
        } else {
            // contract does not support approval management
            return false;
        };

        let approved_account_ids = if let Some(ids) = approvals_by_id.get(&token_id) {
            ids
        } else {
            // token has no approvals
            return false;
        };

        let actual_approval_id = if let Some(id) = approved_account_ids.get(&approved_account_id) {
            id
        } else {
            // account not in approvals HashMap
            return false;
        };

        if let Some(given_approval_id) = approval_id {
            &given_approval_id == actual_approval_id
        } else {
            // account approved, no approval_id given
            true
        }
    }
}
