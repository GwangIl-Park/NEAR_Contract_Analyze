use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, TreeMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::{
    assert_one_yocto, env, log, require, AccountId, BorshStorageKey, Gas, IntoStorageKey,
    PromiseOrValue, PromiseResult, StorageUsage,
};
use nep_171::core::NonFungibleTokenCore;
use nep_171::receiver::ext_nft_receiver;
use nep_171::resolver::{ext_nft_resolver, NonFungibleTokenResolver};
use nep_171::token::{Token, TokenId};
use nep_171::utils::{refund_approved_account_ids, refund_deposit_to_account};
use nep_177::TokenMetadata;
use nep_297::nep_171::{NftMint, NftTransfer};
use std::collections::HashMap;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(5_000_000_000_000);
const GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NonFungibleToken {
    pub owner_id: AccountId,

    pub extra_storage_in_bytes_per_token: StorageUsage,

    pub owner_by_id: TreeMap<TokenId, AccountId>,

    // NEP-177
    pub token_metadata_by_id: Option<LookupMap<TokenId, TokenMetadata>>,

    // NEP-181
    pub tokens_per_owner: Option<LookupMap<AccountId, UnorderedSet<TokenId>>>,

    // NEP-178
    pub approvals_by_id: Option<LookupMap<TokenId, HashMap<AccountId, u64>>>,

    //NEP-178
    pub next_approval_id_by_id: Option<LookupMap<TokenId, u64>>,

    //NEP-199
    pub royalty_by_id: Option<LookupMap<TokenId, HashMap<AccountId, U128>>>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner { account_hash: Vec<u8> },
}

impl NonFungibleToken {
    //prefix가 있는 변수만 생성됨
    pub fn new<Q, R, S, T, U>(
        owner_by_id_prefix: Q,
        owner_id: AccountId,
        token_metadata_prefix: Option<R>,
        enumeration_prefix: Option<S>,
        approval_prefix: Option<T>,
        royalty_prefix: Option<U>,
    ) -> Self
    where
        Q: IntoStorageKey,
        R: IntoStorageKey,
        S: IntoStorageKey,
        T: IntoStorageKey,
        U: IntoStorageKey,
    {
        let (approvals_by_id, next_approval_id_by_id) = if let Some(prefix) = approval_prefix {
            let prefix: Vec<u8> = prefix.into_storage_key();
            (
                Some(LookupMap::new(prefix.clone())),
                Some(LookupMap::new([prefix, "n".into()].concat())),
            )
        } else {
            (None, None)
        };

        let mut this = Self {
            owner_id,
            extra_storage_in_bytes_per_token: 0,
            owner_by_id: TreeMap::new(owner_by_id_prefix),
            token_metadata_by_id: token_metadata_prefix.map(LookupMap::new),
            tokens_per_owner: enumeration_prefix.map(LookupMap::new),
            approvals_by_id,
            next_approval_id_by_id,
            royalty_by_id: royalty_prefix.map(LookupMap::new),
        };
        this.measure_min_token_storage_cost();
        this
    }

    // TODO: does this seem reasonable?
    // 실제로 사용하진 않음
    fn measure_min_token_storage_cost(&mut self) {
        let initial_storage_usage = env::storage_usage();
        // 64 Length because this is the max account id length
        let tmp_token_id = "a".repeat(64);
        let tmp_owner_id = AccountId::new_unchecked("a".repeat(64));

        // 1. set some dummy data
        self.owner_by_id.insert(&tmp_token_id, &tmp_owner_id);
        if let Some(token_metadata_by_id) = &mut self.token_metadata_by_id {
            token_metadata_by_id.insert(
                &tmp_token_id,
                &TokenMetadata {
                    title: Some("a".repeat(64)),
                    description: Some("a".repeat(64)),
                    media: Some("a".repeat(64)),
                    media_hash: Some(Base64VecU8::from("a".repeat(64).as_bytes().to_vec())),
                    copies: Some(1),
                    issued_at: None,
                    expires_at: None,
                    starts_at: None,
                    updated_at: None,
                    extra: None,
                    reference: None,
                    reference_hash: None,
                },
            );
        }
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            let u = &mut UnorderedSet::new(StorageKey::TokensPerOwner {
                account_hash: env::sha256(tmp_owner_id.as_bytes()),
            });
            u.insert(&tmp_token_id);
            tokens_per_owner.insert(&tmp_owner_id, u);
        }
        if let Some(approvals_by_id) = &mut self.approvals_by_id {
            let mut approvals = HashMap::new();
            approvals.insert(tmp_owner_id.clone(), 1u64);
            approvals_by_id.insert(&tmp_token_id, &approvals);
        }
        if let Some(next_approval_id_by_id) = &mut self.next_approval_id_by_id {
            next_approval_id_by_id.insert(&tmp_token_id, &1u64);
        }

        // 2. see how much space it took
        self.extra_storage_in_bytes_per_token = env::storage_usage() - initial_storage_usage;

        // 3. roll it all back
        if let Some(next_approval_id_by_id) = &mut self.next_approval_id_by_id {
            next_approval_id_by_id.remove(&tmp_token_id);
        }
        if let Some(approvals_by_id) = &mut self.approvals_by_id {
            approvals_by_id.remove(&tmp_token_id);
        }
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            let mut u = tokens_per_owner.remove(&tmp_owner_id).unwrap();
            u.remove(&tmp_token_id);
        }
        if let Some(token_metadata_by_id) = &mut self.token_metadata_by_id {
            token_metadata_by_id.remove(&tmp_token_id);
        }
        self.owner_by_id.remove(&tmp_token_id);
    }

    //token_id에 해당하는 토큰을 from으로부터 to에게 전송한다.
    //safety체크나 어떤 로킹도 하지 않는다.
    pub fn internal_transfer_unguarded(
        &mut self,
        token_id: &TokenId,
        from: &AccountId,
        to: &AccountId,
    ) {
        //token_id의 소유자를 to로 변경
        self.owner_by_id.insert(token_id, to);

        //NEP-181 (enumeration 지원할 경우), from에서 token을 삭제하고 to에 추가한다.
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            //from의 토큰들
            let mut owner_tokens = tokens_per_owner.get(from).unwrap_or_else(|| {
                env::panic_str("Unable to access tokens per owner in unguarded call.")
            });
            ///from의 토큰들에서 token_id삭제
            owner_tokens.remove(token_id);
            if owner_tokens.is_empty() {
                tokens_per_owner.remove(from);
            } else {
                tokens_per_owner.insert(from, &owner_tokens);
            }

            let mut receiver_tokens = tokens_per_owner.get(to).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(to.as_bytes()),
                })
            });
            //to의 토큰들에서 token_id추가하고 토큰들에 추가
            receiver_tokens.insert(token_id);
            tokens_per_owner.insert(to, &receiver_tokens);
        }
    }

    //owner로부터 receiver로 전송한다. sender가 전송이 허용되는지 확인한다.
    //approval이 사용되었으면 지운다.
    //이전 소유자와 승인을 리턴한다.
    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> (AccountId, Option<HashMap<AccountId, u64>>) {
        //token_id 소유자
        let owner_id = self
            .owner_by_id
            .get(token_id)
            .unwrap_or_else(|| env::panic_str("Token not found"));

        // token_id의 approve를 삭제한다. 전송이 실패하면 롤백된다.
        let approved_account_ids = self
            .approvals_by_id
            .as_mut()
            .and_then(|by_id| by_id.remove(token_id));

        // sender가 owner가 아닌 경우, approve체크
        let sender_id = if sender_id != &owner_id {
            // approve extension 체크
            let app_acc_ids = approved_account_ids
                .as_ref()
                .unwrap_or_else(|| env::panic_str("Unauthorized"));

            // approve account
            let actual_approval_id = app_acc_ids.get(sender_id);

            // Panic if sender not approved at all
            if actual_approval_id.is_none() {
                env::panic_str("Sender not approved");
            }

            // argument approveid가 sender의 accountID와 같은지 확인
            require!(
                approval_id.is_none() || actual_approval_id == approval_id.as_ref(),
                format!(
                    "The actual approval_id {:?} is different from the given approval_id {:?}",
                    actual_approval_id, approval_id
                )
            );
            Some(sender_id)
        } else {
            None
        };

        require!(
            &owner_id != receiver_id,
            "Current and next owner must differ"
        );

        self.internal_transfer_unguarded(token_id, &owner_id, receiver_id);

        NonFungibleToken::emit_transfer(&owner_id, receiver_id, token_id, sender_id, memo);

        // return previous owner & approvals
        (owner_id, approved_account_ids)
    }

    fn emit_transfer(
        owner_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &str,
        sender_id: Option<&AccountId>,
        memo: Option<String>,
    ) {
        // log!(
        //     r##"EVENT_JSON:{"standard": "nep171", "version": "1.0.0", "event": "nft_transfer", "data":{"old_owner_id": "{}", "new_owner_id": "{}", "token_ids": "{}", "authorized_id": "{}", "memo": "{}"}}"##,
        //     owner_id,
        //     receiver_id,
        //     &[token_id],
        //     sender_id.filter(|sender_id| *sender_id == owner_id),
        //     memo.as_deref()
        // );
        NftTransfer {
            old_owner_id: owner_id,
            new_owner_id: receiver_id,
            token_ids: &[token_id],
            authorized_id: sender_id.filter(|sender_id| *sender_id == owner_id),
            memo: memo.as_deref(),
        }
        .emit();
    }

    /// 표준은 아니다.
    /// `nft_mint` 함수로 쌓여서 처리된다.
    ///
    /// Requirements:
    /// * caller는 컨트랙트 소유자여야 한다.
    /// * caller는 최소 1yoctoNEAR를 첨부해야 한다.
    /// * metadata extension을 사용한다면 metadata를 주어야 한다.
    /// * token_id는 유일해야 한다
    ///
    /// 새로 민팅된 토큰을 리턴한다.
    #[deprecated(
        since = "4.0.0",
        note = "mint is deprecated, please use internal_mint instead."
    )]
    pub fn mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: Option<TokenMetadata>,
        royalties: Option<HashMap<AccountId, U128>>,
    ) -> Token {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Unauthorized");

        self.internal_mint(token_id, token_owner_id, token_metadata, royalties)
    }

    /// 체크하지 않고 민팅한다.:
    /// * caller는 owner_id여야 한다.
    /// * 호출자가 storage비용을 cover하고 남은 비용을 refund해야 한다.
    ///
    /// 새로운 민팅된 토큰을 리턴하고 event를 emit한다.
    pub fn internal_mint(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: Option<TokenMetadata>,
        royalties: Option<HashMap<AccountId, U128>>,
    ) -> Token {
        let token = self.internal_mint_with_refund(
            token_id,
            token_owner_id,
            token_metadata,
            Some(env::predecessor_account_id()),
            royalties,
        );
        NftMint {
            owner_id: &token.owner_id,
            token_ids: &[&token.token_id],
            memo: None,
        }
        .emit();
        token
    }

    /// 체크없이 민팅한다.:
    /// * caller는 owner_id여야 한다.
    /// *
    /// * `refund_id`는 남은 비용을 전송할 account이다.
    ///   일반적으로 owner이다. None이면 환불하지 않는다. 여러 토큰이 발행될 때까지 환불을 지연시키는데 유용하다.
    ///
    /// 민팅 토큰을 리턴하고 mint이벤트를 발생시키지 않는다. 이를 통해 이벤트 전에 여러 민팅이 될 수 있다.
    pub fn internal_mint_with_refund(
        &mut self,
        token_id: TokenId,
        token_owner_id: AccountId,
        token_metadata: Option<TokenMetadata>,
        refund_id: Option<AccountId>,
        royalties: Option<HashMap<AccountId, U128>>,
    ) -> Token {
        // Remember current storage usage if refund_id is Some
        let initial_storage_usage = refund_id.map(|account_id| (account_id, env::storage_usage()));

        if self.token_metadata_by_id.is_some() && token_metadata.is_none() {
            env::panic_str("Must provide metadata");
        }
        if self.owner_by_id.get(&token_id).is_some() {
            env::panic_str("token_id must be unique");
        }

        let owner_id: AccountId = token_owner_id;

        // Core behavior: every token must have an owner
        self.owner_by_id.insert(&token_id, &owner_id);

        // metadata_extension
        self.token_metadata_by_id
            .as_mut()
            .and_then(|by_id| by_id.insert(&token_id, token_metadata.as_ref().unwrap()));

        // Enumeration extension
        if let Some(tokens_per_owner) = &mut self.tokens_per_owner {
            let mut token_ids = tokens_per_owner.get(&owner_id).unwrap_or_else(|| {
                UnorderedSet::new(StorageKey::TokensPerOwner {
                    account_hash: env::sha256(owner_id.as_bytes()),
                })
            });
            token_ids.insert(&token_id);
            tokens_per_owner.insert(&owner_id, &token_ids);
        }

        // Approval Management extension
        let approved_account_ids = if self.approvals_by_id.is_some() {
            Some(HashMap::new())
        } else {
            None
        };

        let payout = if self.royalty_by_id.is_some() {
            Some(royalties.unwrap_or(HashMap::new()))
        } else {
            None
        };

        //남은 비용 refund
        if let Some((id, storage_usage)) = initial_storage_usage {
            refund_deposit_to_account(env::storage_usage() - storage_usage, id)
        }

        // Return any extra attached deposit not used for storage

        Token {
            token_id,
            owner_id,
            metadata: token_metadata,
            approved_account_ids,
            payout,
        }
    }
}

impl NonFungibleTokenCore for NonFungibleToken {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let sender_id = env::predecessor_account_id();
        self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);
    }

    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();
        require!(
            env::prepaid_gas() > GAS_FOR_NFT_TRANSFER_CALL,
            "More gas is required"
        );
        let sender_id = env::predecessor_account_id();
        let (old_owner, old_approvals) =
            self.internal_transfer(&sender_id, &receiver_id, &token_id, approval_id, memo);
        // Initiating receiver's call and the callback
        ext_nft_receiver::ext(receiver_id.clone())
            .with_static_gas(env::prepaid_gas() - GAS_FOR_NFT_TRANSFER_CALL)
            .nft_on_transfer(sender_id, old_owner.clone(), token_id.clone(), msg)
            .then(
                ext_nft_resolver::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                    .nft_resolve_transfer(old_owner, receiver_id, token_id, old_approvals),
            )
            .into()
    }

    fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        let owner_id = self.owner_by_id.get(&token_id)?;
        let metadata = self
            .token_metadata_by_id
            .as_ref()
            .and_then(|by_id| by_id.get(&token_id));
        let approved_account_ids = self
            .approvals_by_id
            .as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));
        let payout = self
            .royalty_by_id
            .as_ref()
            .and_then(|by_id| by_id.get(&token_id).or_else(|| Some(HashMap::new())));
        Some(Token {
            token_id,
            owner_id,
            metadata,
            approved_account_ids,
            payout,
        })
    }
}

impl NonFungibleTokenResolver for NonFungibleToken {
    /// Returns true if token was successfully transferred to `receiver_id`.
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        // Get whether token should be returned
        let must_revert = match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(value) => {
                if let Ok(yes_or_no) = near_sdk::serde_json::from_slice::<bool>(&value) {
                    yes_or_no
                } else {
                    true
                }
            }
            PromiseResult::Failed => true,
        };

        // 성공이면 리턴
        if !must_revert {
            return true;
        }

        // OTHERWISE, try to set owner back to previous_owner_id and restore approved_account_ids

        // receiver가 이미 전송해버렸는지 혹은 소각했는지 확인
        if let Some(current_owner) = self.owner_by_id.get(&token_id) {
            if current_owner != receiver_id {
                return true;
            }
        } else {
            // 토큰이 소각되어 버림
            // 소각되었으므로 스토리지 비용 환불
            if let Some(approved_account_ids) = approved_account_ids {
                refund_approved_account_ids(previous_owner_id, &approved_account_ids);
            }
            return true;
        };

        self.internal_transfer_unguarded(&token_id, &receiver_id, &previous_owner_id);

        // Approval Management extension를 사용하고 있다면
        if let Some(by_id) = &mut self.approvals_by_id {
            // 스토리지 비용 환불
            if let Some(receiver_approvals) = by_id.get(&token_id) {
                refund_approved_account_ids(receiver_id.clone(), &receiver_approvals);
            }
            // 이전 approval로 돌려놓는다.
            if let Some(previous_owner_approvals) = approved_account_ids {
                by_id.insert(&token_id, &previous_owner_approvals);
            }
        }
        NonFungibleToken::emit_transfer(&receiver_id, &previous_owner_id, &token_id, None, None);
        false
    }
}
