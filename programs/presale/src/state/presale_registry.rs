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
    /// This is the minimum amount of quote token that a user can deposit to the presale. Personal cap must within the global cap range.
    pub buyer_minimum_deposit_cap: u64,
    /// This is the maximum amount of quote token that a user can deposit to the presale. Personal cap must within the global cap range.
    pub buyer_maximum_deposit_cap: u64,
    /// Deposit fee collected
    pub total_deposit_fee: u64,
    /// Deposit fee bps
    pub deposit_fee_bps: u16,
    pub padding0: [u8; 14],
    pub padding1: [u128; 5],
}

static_assertions::const_assert_eq!(PresaleRegistry::INIT_SPACE, 160);
static_assertions::assert_eq_align!(PresaleRegistry, u128);

impl PresaleRegistry {
    pub fn init(
        &mut self,
        presale_supply: u64,
        buyer_minimum_deposit_cap: u64,
        buyer_maximum_deposit_cap: u64,
        deposit_fee_bps: u16,
    ) {
        self.presale_supply = presale_supply;
        self.buyer_minimum_deposit_cap = buyer_minimum_deposit_cap;
        self.buyer_maximum_deposit_cap = buyer_maximum_deposit_cap;
        self.deposit_fee_bps = deposit_fee_bps;
    }

    pub fn calculate_deposit_fee_included_amount(
        &self,
        deposit_amount: u64,
    ) -> Result<DepositFeeIncludedCalculation> {
        calculate_deposit_fee_included_amount(deposit_amount, self.deposit_fee_bps, Rounding::Up)
    }

    pub fn deposit(
        &mut self,
        escrow: &mut Escrow,
        fee_excluded_deposit_amount: u64,
        fee: u64,
    ) -> Result<()> {
        self.total_deposit = self.total_deposit.safe_add(fee_excluded_deposit_amount)?;
        self.total_deposit_fee = self.total_deposit_fee.safe_add(fee)?;

        escrow.deposit(fee_excluded_deposit_amount, fee)?;
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

    pub fn is_uninitialized(&self) -> bool {
        self.presale_supply == 0
            && self.buyer_maximum_deposit_cap == 0
            && self.buyer_minimum_deposit_cap == 0
            && self.deposit_fee_bps == 0
    }

    pub fn get_remaining_quote(
        &self,
        presale_remaining_quote: u64,
        presale_total_deposit: u64,
    ) -> Result<RemainingQuote> {
        if presale_total_deposit == 0 || self.total_deposit == 0 {
            return Ok(RemainingQuote {
                refund_amount: 0,
                refund_fee: 0,
            });
        }

        let registry_remaining_quote = u128::from(presale_remaining_quote)
            .safe_mul(self.total_deposit.into())?
            .safe_div(presale_total_deposit.into())?;

        let registry_refund_fee = u128::from(self.total_deposit_fee)
            .safe_mul(registry_remaining_quote)?
            .safe_div(self.total_deposit.into())?;

        Ok(RemainingQuote {
            refund_amount: registry_remaining_quote.safe_cast()?,
            refund_fee: registry_refund_fee.safe_cast()?,
        })
    }

    pub fn ensure_escrow_deposit_within_cap(&self, escrow: &Escrow) -> Result<()> {
        if escrow.total_deposit == 0 {
            return Ok(());
        }

        require!(
            escrow.total_deposit >= self.buyer_minimum_deposit_cap
                && escrow.total_deposit <= self.buyer_maximum_deposit_cap,
            PresaleError::DepositAmountOutOfCap
        );

        Ok(())
    }
}

pub struct RemainingQuote {
    pub refund_amount: u64,
    pub refund_fee: u64,
}
