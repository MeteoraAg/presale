use crate::*;

#[event]
pub struct EventFixedPricePresaleParams {
    pub presale: Pubkey,
    pub unsold_token_action: u8,
    pub q_price: u128,
}
