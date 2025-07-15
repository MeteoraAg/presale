use crate::*;

// TODO: Assert account size to changes on padding have no effect on the account size.

#[account(zero_copy)]
#[derive(Debug, InitSpace)]
pub struct Escrow {
    // Presale vault of the escrow
    pub presale: Pubkey,
    // The owner of the escrow
    pub owner: Pubkey,
    // Total deposited quote token
    pub total_deposit: u64,
    // Deposit fee
    pub deposit_fee: u64,
    // Total claimed base token
    pub total_claimed_token: u64,
    // Determine whether user withdrawn remaining quote token
    pub is_remaining_quote_withdrawn: u8,
    pub padding0: [u8; 7],
    // Total pending claim token
    pub pending_claim_token: u64,
    // Timestamp of when the escrow was created
    pub created_at: u64,
    // Timestamp of when the escrow was refreshed
    pub last_refreshed_at: u64,
}

impl Escrow {
    pub fn initialize(&mut self, presale: Pubkey, owner: Pubkey, created_at: u64) -> Result<()> {
        self.presale = presale;
        self.owner = owner;
        self.total_deposit = 0;
        self.created_at = created_at;
        self.last_refreshed_at = created_at;

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

    pub fn withdraw(&mut self, amount: u64) -> Result<u64> {
        // TODO: Test this whether if repeatly deposit and withdraw will causes the amount + fee > reserve amount
        let mut fee_amount = self
            .deposit_fee
            .checked_mul(amount)
            .unwrap()
            .checked_div(self.total_deposit)
            .unwrap();

        self.total_deposit = self.total_deposit.checked_sub(amount).unwrap();

        // Withdraw all
        if self.total_deposit == 0 {
            fee_amount = self.deposit_fee;
        }

        self.deposit_fee = self.deposit_fee.checked_sub(fee_amount).unwrap();

        Ok(fee_amount)
    }

    pub fn claim(&mut self) -> Result<()> {
        self.total_claimed_token = self
            .total_claimed_token
            .checked_add(self.pending_claim_token)
            .unwrap();
        self.pending_claim_token = 0;
        Ok(())
    }

    pub fn update_remaining_quote_withdrawn(&mut self) -> Result<()> {
        self.is_remaining_quote_withdrawn = 1;
        Ok(())
    }

    pub fn is_remaining_quote_withdrawn(&self) -> bool {
        self.is_remaining_quote_withdrawn == 1
    }

    pub fn get_total_deposit_amount_with_fees(&self) -> Result<u64> {
        let total_deposit_with_fees = self.total_deposit.checked_add(self.deposit_fee).unwrap();
        Ok(total_deposit_with_fees)
    }

    pub fn sum_claimed_and_pending_claim_amount(&self) -> Result<u64> {
        Ok(self
            .total_claimed_token
            .checked_add(self.pending_claim_token)
            .unwrap())
    }

    pub fn accumulate_pending_claim_token(&mut self, pending_claim_token: u64) -> Result<()> {
        self.pending_claim_token = self
            .pending_claim_token
            .checked_add(pending_claim_token)
            .unwrap();
        Ok(())
    }

    pub fn update_last_refreshed_at(&mut self, current_timestamp: u64) -> Result<()> {
        self.last_refreshed_at = current_timestamp;
        Ok(())
    }
}
