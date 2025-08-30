use crate::*;

#[account(zero_copy)]
#[derive(Debug, InitSpace)]
pub struct Escrow {
    // Presale vault of the escrow
    pub presale: Pubkey,
    // The owner of the escrow
    pub owner: Pubkey,
    // Total deposited quote token
    pub total_deposit: u64,
    // Total claimed base token
    pub total_claimed_token: u64,
    // Determine whether user withdrawn remaining quote token
    pub is_remaining_quote_withdrawn: u8,
    // The index of the presale registry
    pub registry_index: u8,
    pub padding0: [u8; 6],
    // Total pending claim token
    pub pending_claim_token: u64,
    // Personal deposit cap. Only available if whitelist mode is permissioned.
    pub deposit_max_cap: u64,
    // Timestamp of when the escrow was created
    pub created_at: u64,
    // Total deposit fee collected
    pub total_deposit_fee: u64,
    // Timestamp of when the escrow was refreshed
    pub last_refreshed_at: u64,
    pub padding: [u64; 8],
}

static_assertions::const_assert_eq!(Escrow::INIT_SPACE, 192);
static_assertions::assert_eq_align!(Escrow, u64);

impl Escrow {
    pub fn initialize(
        &mut self,
        presale: Pubkey,
        owner: Pubkey,
        created_at: u64,
        registry_index: u8,
        deposit_cap: u64,
    ) -> Result<()> {
        self.presale = presale;
        self.owner = owner;
        self.total_deposit = 0;
        self.created_at = created_at;
        self.last_refreshed_at = created_at;
        self.registry_index = registry_index;
        self.deposit_max_cap = deposit_cap;

        Ok(())
    }

    pub fn get_remaining_deposit_quota(&self, buyer_maximum_buy_cap: u64) -> Result<u64> {
        let maximum_buy_cap = buyer_maximum_buy_cap.min(self.deposit_max_cap);
        if self.total_deposit >= maximum_buy_cap {
            return Ok(0);
        }

        let remaining_quota = maximum_buy_cap.safe_sub(self.total_deposit)?;
        Ok(remaining_quota)
    }

    pub fn deposit(&mut self, fee_excluded_deposit_amount: u64, fee: u64) -> Result<()> {
        self.total_deposit = self.total_deposit.safe_add(fee_excluded_deposit_amount)?;
        self.total_deposit_fee = self.total_deposit_fee.safe_add(fee)?;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        self.total_deposit = self.total_deposit.safe_sub(amount)?;
        Ok(())
    }

    pub fn claim(&mut self) -> Result<u64> {
        self.total_claimed_token = self
            .total_claimed_token
            .safe_add(self.pending_claim_token)?;
        let claimed_token = self.pending_claim_token;
        self.pending_claim_token = 0;
        Ok(claimed_token)
    }

    pub fn update_remaining_quote_withdrawn(&mut self) -> Result<()> {
        self.is_remaining_quote_withdrawn = 1;
        Ok(())
    }

    pub fn is_remaining_quote_withdrawn(&self) -> bool {
        self.is_remaining_quote_withdrawn == 1
    }

    pub fn sum_claimed_and_pending_claim_amount(&self) -> Result<u64> {
        Ok(self
            .total_claimed_token
            .safe_add(self.pending_claim_token)?)
    }

    pub fn accumulate_pending_claim_token(&mut self, pending_claim_token: u64) -> Result<()> {
        self.pending_claim_token = self.pending_claim_token.safe_add(pending_claim_token)?;
        Ok(())
    }

    pub fn update_last_refreshed_at(&mut self, current_timestamp: u64) -> Result<()> {
        self.last_refreshed_at = current_timestamp;
        Ok(())
    }
}
