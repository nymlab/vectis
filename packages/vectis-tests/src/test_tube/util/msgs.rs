use crate::constants::DENOM;
use cosmwasm_std::{coin, BankMsg, CosmosMsg, Empty};

pub fn simple_bank_send() -> CosmosMsg {
    CosmosMsg::<Empty>::Bank(BankMsg::Send {
        to_address: "osmo1pkf6nuq8whw5ta5537c3uqrep0yzcwkrw82n95".into(),
        amount: vec![coin(1, DENOM)],
    })
}
