use crate::*;

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct FixedPricePresaleExtraArgs {
    pub padding0: [u8; 16],
    pub q_price: u128,
    pub owner: Pubkey,
    pub presale: Pubkey,
    pub padding1: [u128; 4],
}

static_assertions::const_assert_eq!(FixedPricePresaleExtraArgs::INIT_SPACE, 160);
static_assertions::assert_eq_align!(FixedPricePresaleExtraArgs, u128);

impl FixedPricePresaleExtraArgs {
    fn validate(q_price: u128) -> Result<()> {
        require!(q_price > 0, PresaleError::InvalidTokenPrice);
        Ok(())
    }

    pub fn validate_and_initialize(
        &mut self,
        q_price: u128,
        owner: Pubkey,
        presale: Pubkey,
    ) -> Result<()> {
        Self::validate(q_price)?;

        self.q_price = q_price;
        self.owner = owner;
        self.presale = presale;

        Ok(())
    }
}
