use crate::*;
use num_enum::{FromPrimitive, IntoPrimitive};

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct FixedPricePresaleExtraArgs {
    pub padding0: [u8; 15],
    pub disable_withdraw: u8,
    pub q_price: u128,
    pub owner: Pubkey,
    pub presale: Pubkey,
    pub padding1: [u128; 4],
}

static_assertions::const_assert_eq!(FixedPricePresaleExtraArgs::INIT_SPACE, 160);
static_assertions::assert_eq_align!(FixedPricePresaleExtraArgs, u128);

impl FixedPricePresaleExtraArgs {
    pub fn initialize(
        &mut self,
        q_price: u128,
        owner: Pubkey,
        presale: Pubkey,
        disable_withdraw: BoolType,
    ) -> Result<()> {
        self.q_price = q_price;
        self.owner = owner;
        self.presale = presale;
        self.disable_withdraw = disable_withdraw.into();

        Ok(())
    }
}
