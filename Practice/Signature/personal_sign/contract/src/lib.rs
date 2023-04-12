use std::convert::TryFrom;

use ed25519_dalek::Verifier;
use near_sdk::base64::decode as decode64;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::bs58::decode as decode58;
use near_sdk::{log, near_bindgen};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct VerifyContract {}

#[near_bindgen]
impl VerifyContract {
    pub fn verify(&mut self, message_hash: &[u8; 32], public_key_param: String, signature: String) {
        let signature =
            ed25519_dalek::Signature::try_from(decode64(signature).unwrap().as_ref()).unwrap();
        // let public_key2 = near_sdk::env::signer_account_pk();
        // let (_, public_key_32) = public_key2.as_bytes().split_at(1);
        // let public_key = ed25519_dalek::PublicKey::from_bytes(public_key_32).unwrap();

        let public_key =
            ed25519_dalek::PublicKey::from_bytes(&decode58(public_key_param).into_vec().unwrap())
                .unwrap();

        if let Ok(_) = public_key.verify(message_hash, &signature) {
            log!("success")
        } else {
            log!("fail");
        }
    }
}
