use crate::*;

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct Operator {
    pub owner: Pubkey,
    pub creator: Pubkey,
    pub padding: [u64; 4],
}

impl Operator {
    pub fn initialize(&mut self, owner: Pubkey, creator: Pubkey) {
        self.owner = owner;
        self.creator = creator;
    }
}
