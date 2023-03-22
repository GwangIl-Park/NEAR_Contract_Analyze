use std::convert::TryFrom;

use ed25519_dalek::Verifier;

use near_sdk::base64::decode as decode64;
use near_sdk::base64::encode as encode64;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::bs58::decode as decode58;
use near_sdk::env::sha256_array;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{log, near_bindgen, AccountId};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
struct EIP712Domain {
    name: String,
    version: String,
    chain_id: String,
    verifying_contract: AccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Person {
    name: String,
    wallet: AccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Mail {
    from: Person,
    to: Person,
    contents: String,
}

#[derive(Serialize, Deserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payload {
    tag: u32,
    message: [u8; 32],
    nonce: [u8; 32],
    receipient: AccountId,
    callbackUrl: String,
}

//sha256_array("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)");
const EIP712DOMAIN_TYPEHASH: [u8; 32] = [
    198, 59, 48, 83, 117, 35, 182, 76, 53, 33, 226, 87, 66, 137, 235, 230, 230, 134, 35, 176, 229,
    204, 101, 238, 119, 187, 26, 87, 155, 227, 67, 239,
];

//sha256_array("Person(string name,address wallet)");
const PERSON_TYPEHASH: [u8; 32] = [
    11, 175, 250, 237, 110, 96, 120, 229, 28, 102, 40, 124, 59, 106, 84, 249, 87, 238, 156, 1, 219,
    126, 140, 71, 170, 107, 179, 159, 22, 4, 138, 213,
];

//sha256_array("Mail(Person from,Person to,string contents)Person(string name,address wallet)");
const MAIL_TYPEHASH: [u8; 32] = [
    123, 96, 125, 176, 215, 143, 52, 177, 90, 173, 182, 32, 26, 61, 220, 53, 155, 135, 6, 4, 126,
    196, 195, 186, 217, 183, 8, 200, 246, 202, 98, 175,
];
// pub fn test(&self) {
//     let bb = env::current_account_id();
//     let input: Vec<u8> = [
//         EIP712DOMAIN_TYPEHASH.as_ref(),
//         sha256_array("NEARMail".as_bytes()).as_ref(),
//         sha256_array("1".as_bytes()).as_ref(),
//         sha256_array("testnet".as_bytes()).as_ref(),
//         sha256_array(bb.as_bytes()).as_ref(),
//     ]
//     .concat();
//     let mut input2 = vec![];
//     BorshSerialize::serialize(&input, &mut input2);
//     let aa = sha256_array(&input2);
//     log!("{:?}", aa)
// }
const DOMAIN_SEPARATOR: [u8; 32] = [
    236, 186, 149, 206, 28, 208, 103, 29, 234, 94, 209, 25, 153, 238, 4, 68, 253, 236, 78, 68, 86,
    169, 6, 113, 51, 216, 182, 56, 114, 196, 230, 89,
];

const NONCE: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];

const PREFIX_TAG: u32 = 2147484061;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct EIP712 {}

#[near_bindgen]
impl EIP712 {
    fn hash_person(&self, person: &Person) -> [u8; 32] {
        let mut encoded_input = vec![];
        let input = [
            PERSON_TYPEHASH,
            sha256_array(person.name.as_bytes()),
            sha256_array(person.wallet.as_bytes()),
        ]
        .concat();
        BorshSerialize::serialize(&input, &mut encoded_input);
        sha256_array(&encoded_input)
    }

    fn hash_mail(&self, mail: &Mail) -> [u8; 32] {
        let mut encoded_input = vec![];
        let input = [
            MAIL_TYPEHASH,
            self.hash_person(&mail.from),
            self.hash_person(&mail.to),
            sha256_array(mail.contents.as_bytes()),
        ]
        .concat();
        BorshSerialize::serialize(&input, &mut encoded_input);
        sha256_array(&encoded_input)
    }

    fn hash_message(&self, mail: &Mail) -> [u8; 32] {
        let mut encoded_input = vec![];
        let input = [DOMAIN_SEPARATOR, self.hash_mail(mail)].concat();
        BorshSerialize::serialize(&input, &mut encoded_input);
        sha256_array(&encoded_input)
    }

    pub fn verify(
        &mut self,
        mail: Mail,
        public_key_param: String,
        signature: String,
        receipient: AccountId,
    ) {
        let signature =
            ed25519_dalek::Signature::try_from(decode64(signature).unwrap().as_ref()).unwrap();

        // let public_key2 = near_sdk::env::signer_account_pk();
        // let (_, public_key_32) = public_key2.as_bytes().split_at(1);
        // let public_key = ed25519_dalek::PublicKey::from_bytes(public_key_32).unwrap();

        let public_key =
            ed25519_dalek::PublicKey::from_bytes(&decode58(public_key_param).into_vec().unwrap())
                .unwrap();

        let hash_message = self.hash_message(&mail);

        let payload = Payload {
            tag: PREFIX_TAG,
            message: hash_message,
            nonce: NONCE,
            receipient,
            callbackUrl: "".to_string(),
        };

        let mut encoded_input = vec![];
        BorshSerialize::serialize(&payload, &mut encoded_input);
        sha256_array(&encoded_input);

        if let Ok(_) = public_key.verify(&sha256_array(&encoded_input), &signature) {
            log!("success")
        } else {
            log!("fail");
        }
    }
}
