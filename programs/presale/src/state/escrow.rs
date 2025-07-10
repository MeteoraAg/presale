use crate::*;

// TODO: Assert account size to changes on padding have no effect on the account size.

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct Escrow {
    // Presale vault of the escrow
    pub presale: Pubkey,
    // The owner of the escrow
    pub owner: Pubkey,
    // Total deposited quote token
    pub total_deposit: u64,
    // Timestamp of when the escrow was created
    pub created_at: u64,
}

impl Escrow {
    pub fn initialize(&mut self, presale: Pubkey, owner: Pubkey, created_at: u64) -> Result<()> {
        self.presale = presale;
        self.owner = owner;
        self.total_deposit = 0;
        self.created_at = created_at;

        Ok(())
    }
}
