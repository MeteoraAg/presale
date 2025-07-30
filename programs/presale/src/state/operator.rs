use crate::*;

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct Operator {
    pub owner: Pubkey,
    pub creator: Pubkey,
    pub padding: [u64; 8],
}

static_assertions::const_assert_eq!(Operator::INIT_SPACE, 128);
static_assertions::assert_eq_align!(Operator, u64);

impl Operator {
    pub fn initialize(&mut self, owner: Pubkey, creator: Pubkey) {
        self.owner = owner;
        self.creator = creator;
    }
}
