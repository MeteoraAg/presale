use crate::*;

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct FixedPricePresaleExtraArgs {
    pub unsold_token_action: u8,
    pub padding0: [u8; 15],
    pub q_price: u128,
    pub owner: Pubkey,
    pub presale: Pubkey,
    pub padding1: [u128; 4],
}

impl FixedPricePresaleExtraArgs {
    fn validate(unsold_token_action: u8, q_price: u128) -> Result<()> {
        UnsoldTokenAction::try_from(unsold_token_action).unwrap();
        require!(q_price > 0, PresaleError::InvalidTokenPrice);

        Ok(())
    }

    pub fn validate_and_initialize(
        &mut self,
        unsold_token_action: u8,
        q_price: u128,
        owner: Pubkey,
        presale: Pubkey,
    ) -> Result<()> {
        Self::validate(unsold_token_action, q_price).unwrap();
        self.unsold_token_action = unsold_token_action;
        self.q_price = q_price;
        self.owner = owner;
        self.presale = presale;

        Ok(())
    }
}
