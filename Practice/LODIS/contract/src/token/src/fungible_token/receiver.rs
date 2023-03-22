use near_sdk::json_types::U128;
use near_sdk::{ext_contract, AccountId, PromiseOrValue};

//transferAndCall 호출시 호출될 function 연결
#[ext_contract(ext_ft_receiver)]
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<String>;
}
