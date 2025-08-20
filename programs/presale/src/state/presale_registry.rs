use crate::*;

#[zero_copy]
#[derive(InitSpace, Debug, Default)]
pub struct PresaleRegistry {
    /// Total supply of tokens available for this presale registry
    pub presale_supply: u64,
    /// Total amount of tokens deposited in this presale registry
    pub total_deposit: u64,
    /// Total escrow in this presale registry
    pub total_escrow: u64,
    /// Total claimed base token. For statistic purpose only
    pub total_claimed_token: u64,
    /// Total refunded quote token. For statistic purpose only
    pub total_refunded_quote_token: u64,
    /// This is the minimum amount of quote token that a user can deposit to the presale
    pub buyer_minimum_deposit_cap: u64,
    /// This is the maximum amount of quote token that a user can deposit to the presale
    pub buyer_maximum_deposit_cap: u64,
    pub padding0: [u8; 8],
    pub padding1: [u128; 5],
}

static_assertions::const_assert_eq!(PresaleRegistry::INIT_SPACE, 144);
static_assertions::assert_eq_align!(PresaleRegistry, u128);

impl PresaleRegistry {
    pub fn init(
        &mut self,
        presale_supply: u64,
        buyer_minimum_deposit_cap: u64,
        buyer_maximum_deposit_cap: u64,
    ) {
        self.presale_supply = presale_supply;
        self.buyer_minimum_deposit_cap = buyer_minimum_deposit_cap;
        self.buyer_maximum_deposit_cap = buyer_maximum_deposit_cap;
    }

    pub fn deposit(&mut self, escrow: &mut Escrow, deposit_amount: u64) -> Result<()> {
        self.total_deposit = self.total_deposit.safe_add(deposit_amount)?;
        escrow.deposit(deposit_amount)?;
        Ok(())
    }

    pub fn withdraw(&mut self, escrow: &mut Escrow, amount: u64) -> Result<()> {
        escrow.withdraw(amount)?;
        self.total_deposit = self.total_deposit.safe_sub(amount)?;
        Ok(())
    }

    pub fn increase_escrow_count(&mut self) -> Result<()> {
        self.total_escrow = self.total_escrow.safe_add(1)?;
        Ok(())
    }

    pub fn decrease_escrow_count(&mut self) -> Result<()> {
        self.total_escrow = self.total_escrow.safe_sub(1)?;
        Ok(())
    }

    pub fn update_total_refunded_quote_token(&mut self, amount: u64) -> Result<()> {
        self.total_refunded_quote_token = self.total_refunded_quote_token.safe_add(amount)?;
        Ok(())
    }

    pub fn update_total_claim_amount(&mut self, claimed_amount: u64) -> Result<()> {
        self.total_claimed_token = self.total_claimed_token.safe_add(claimed_amount)?;
        Ok(())
    }
}
