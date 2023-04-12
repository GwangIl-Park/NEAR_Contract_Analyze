use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::require;
use near_sdk::serde::{Deserialize, Serialize};

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "nft-1.0.0";

/// Metadata for the NFT contract itself.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema))]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // 추가 정보가 있는 JSON파일의 URL
    pub reference_hash: Option<Base64VecU8>, // reference필드에 있는 JSON의 Base64-encoded sha256해쉬값, reference필드가 있다면 필수
}

/// Metadata on the individual token level.
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, BorshDeserialize, BorshSerialize, Default,
)]
#[cfg_attr(feature = "abi", derive(schemars::JsonSchema))]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // 관련 미디어, 컨텐츠 주소 저장소 URL
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of media 필드
    pub copies: Option<u64>,   // 토큰이 민팅되었을때, 존재한 메타데이터 세트의 복사본 개수
    pub issued_at: Option<String>, // ISO 8601 datetime 토큰이 민팅되었을 때
    pub expires_at: Option<String>, // ISO 8601 datetime 토큰 만료 시간
    pub starts_at: Option<String>, // ISO 8601 datetime 토큰이 유효하기 시작한 시간
    pub updated_at: Option<String>, // ISO 8601 datetime 토큰이 마지막으로 업데이트된 시간
    pub extra: Option<String>, // NFT로 온체인에 저장하고 싶은 추가 데이터, stringified JSON형태
    pub reference: Option<String>, // 추가데이터가 있는 오프체인 JSON파일의 URL
    pub reference_hash: Option<Base64VecU8>, // reference의 Base64-encoded sha256 hash
}

/// Offers details on the contract-level metadata.
pub trait NonFungibleTokenMetadataProvider {
    fn nft_metadata(&self) -> NFTContractMetadata;
}

impl NFTContractMetadata {
    pub fn assert_valid(&self) {
        require!(self.spec == NFT_METADATA_SPEC, "Spec is not NFT metadata");
        require!(
            self.reference.is_some() == self.reference_hash.is_some(),
            "Reference and reference hash must be present"
        );
        if let Some(reference_hash) = &self.reference_hash {
            require!(reference_hash.0.len() == 32, "Hash has to be 32 bytes");
        }
    }
}

impl TokenMetadata {
    pub fn assert_valid(&self) {
        require!(self.media.is_some() == self.media_hash.is_some());
        if let Some(media_hash) = &self.media_hash {
            require!(media_hash.0.len() == 32, "Media hash has to be 32 bytes");
        }

        require!(self.reference.is_some() == self.reference_hash.is_some());
        if let Some(reference_hash) = &self.reference_hash {
            require!(
                reference_hash.0.len() == 32,
                "Reference hash has to be 32 bytes"
            );
        }
    }
}
