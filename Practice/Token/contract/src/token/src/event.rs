use near_sdk::env;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "standard")]
#[must_use = "don't forget to `.emit()` this event"]
#[serde(rename_all = "snake_case")]
pub(crate) enum NearEvent<'a> {
    //pub(crate) : 이 enum 타입이 모듈(crate)내부에서는 공개되지만 모듈 외부에서는 비공개
    //<'a> : 라이프타임 파라미터 : 변수나 참조가 언제 생성되고 소멸하는지 명시
    Nep141(crate::fungible_token::events::Nep141Event<'a>),
    //Nep171(crate::non_fungible_token::events::Nep171Event<'a>),
}

// {
//     "standard":"nep141"
//     "version": "1.0.0",
//     "event":"ft_burn"
//     "data":{
//         "owner_id":"aaa",
//         "amount":"11",
//         "memo":"asdad"
//     }
// }

impl<'a> NearEvent<'a> {
    fn to_json_string(&self) -> String {
        #[allow(clippy::redundant_closure)]
        serde_json::to_string(self) //serialize된 json을 string으로 출력
            .ok()
            .unwrap_or_else(|| env::abort())
    }

    fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }

    pub(crate) fn emit(self) {
        near_sdk::env::log_str(&self.to_json_event_string());
    }
}
