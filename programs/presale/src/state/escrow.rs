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
    // Deposit fee
    pub deposit_fee: u64,
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

    pub fn get_remaining_deposit_quota(&self, buyer_maximum_buy_cap: u64) -> Result<u64> {
        if self.total_deposit >= buyer_maximum_buy_cap {
            return Ok(0);
        }

        let remaining_quota = buyer_maximum_buy_cap
            .checked_sub(self.total_deposit)
            .unwrap();

        Ok(remaining_quota)
    }

    pub fn deposit(&mut self, deposit_fee_excluded_amount: u64, deposit_fee: u64) -> Result<()> {
        self.total_deposit = self
            .total_deposit
            .checked_add(deposit_fee_excluded_amount)
            .unwrap();
        self.deposit_fee = self.deposit_fee.checked_add(deposit_fee).unwrap();

        Ok(())
    }
}
