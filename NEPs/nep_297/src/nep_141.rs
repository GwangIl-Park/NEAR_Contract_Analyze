use crate::event::NearEvent;
use near_sdk::json_types::U128;
use near_sdk::AccountId;
use serde::Serialize;

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct FtMint<'a> {
    pub owner_id: &'a AccountId,
    pub amount: &'a U128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>,
}

impl FtMint<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many(data: &[FtMint<'_>]) {
        new_141_v1(Nep141EventKind::FtMint(data)).emit()
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct FtTransfer<'a> {
    pub old_owner_id: &'a AccountId,
    pub new_owner_id: &'a AccountId,
    pub amount: &'a U128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>,
}

impl FtTransfer<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many(data: &[FtTransfer<'_>]) {
        new_141_v1(Nep141EventKind::FtTransfer(data)).emit()
    }
}

#[must_use]
#[derive(Serialize, Debug, Clone)]
pub struct FtBurn<'a> {
    pub owner_id: &'a AccountId,
    pub amount: &'a U128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<&'a str>,
}

impl FtBurn<'_> {
    pub fn emit(self) {
        Self::emit_many(&[self])
    }

    pub fn emit_many<'a>(data: &'a [FtBurn<'a>]) {
        new_141_v1(Nep141EventKind::FtBurn(data)).emit()
    }
}

#[derive(Serialize, Debug)]
pub(crate) struct Nep141Event<'a> {
    version: &'static str,
    #[serde(flatten)]
    event_kind: Nep141EventKind<'a>,
}

//#[serde(flatten)]
// {
//     "version": "1.0.0",
//     "event_kind":{
//         "event":"ft_burn"
//         "data":{
//             "owner_id":"aaa",
//             "amount":"11",
//             "memo":"asdad"
//         }
//     }
// }
// ->
// {
//     "version": "1.0.0",
//     "event":"ft_burn"
//     "data":{
//         "owner_id":"aaa",
//         "amount":"11",
//         "memo":"asdad"
//     }
// }

#[derive(Serialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)] //열거형을 사용할때 대문자를 사용하는 것이 원칙인데 소문자로 쓰는 것을 허용
enum Nep141EventKind<'a> {
    FtMint(&'a [FtMint<'a>]),
    FtTransfer(&'a [FtTransfer<'a>]),
    FtBurn(&'a [FtBurn<'a>]),
}

fn new_141<'a>(version: &'static str, event_kind: Nep141EventKind<'a>) -> NearEvent<'a> {
    NearEvent::Nep141(Nep141Event {
        version,
        event_kind,
    })
}

fn new_141_v1(event_kind: Nep141EventKind) -> NearEvent {
    new_141("1.0.0", event_kind)
}
