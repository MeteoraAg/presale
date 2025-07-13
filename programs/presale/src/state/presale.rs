use crate::*;
use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum PresaleMode {
    /// Fixed token price. The remaining will either be burn or refund to the creator
    #[num_enum(default)]
    FixedPrice,
    /// Dynamic token price. The price will be determined by how many quote tokens used to buy base tokens
    Prorata,
    /// Dynamic token price. The price will be determined by how many quote tokens used to buy base tokens
    Fcfs,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum WhitelistMode {
    #[num_enum(default)]
    /// No whitelist
    Permissionless,
    /// Whitelist using merkle proof
    PermissionWithMerkleProof,
    /// Whitelist by allowing only vault's creator to create escrow account
    PermissionWithAuthority,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum UnsoldTokenAction {
    /// Refund unsold token back to creator
    #[num_enum(default)]
    Refund,
    /// Burn unsold token
    Burn,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PresaleProgress {
    /// Presale has not started yet
    NotStarted,
    /// Presale is ongoing
    Ongoing,
    /// Presale is ended
    Completed,
    /// Presale is ended but not enough capital raised
    Failed,
}

#[account(zero_copy)]
#[derive(InitSpace, Debug)]
pub struct Presale {
    /// Owner of presale
    pub owner: Pubkey,
    /// Quote token mint
    pub quote_mint: Pubkey,
    /// Base token
    pub base_mint: Pubkey,
    /// Base token vault
    pub base_token_vault: Pubkey,
    /// Quote token vault
    pub quote_token_vault: Pubkey,
    pub padding0: [u8; 32],
    /// Presale target raised capital
    pub presale_maximum_cap: u64,
    /// Presale minimum raised capital. Else, presale consider as failed.
    pub presale_minimum_cap: u64,
    /// When presale starts
    pub presale_start_time: u64,
    /// When presale ends. Presale can be ended earlier by creator if raised capital is reached (based on presale mode)
    pub presale_end_time: u64,
    /// This is the minimum amount of quote token that a user can deposit to the presale
    pub buyer_minimum_deposit_cap: u64,
    /// This is the maximum amount of quote token that a user can deposit to the presale
    pub buyer_maximum_deposit_cap: u64,
    /// Total base token supply that can be bought by presale participants
    pub presale_supply: u64,
    /// Total base token supply reserved for the creator
    pub creator_supply: u64,
    /// Total deposited quote token
    pub total_deposit: u64,
    /// Total deposit fee collected
    pub total_deposit_fee: u64,
    /// Total number of depositors. For statistic purpose only
    pub total_escrow: u64,
    /// Total escrow fee collected. For statistic purpose only
    pub total_escrow_fee: u64,
    /// When was the presale created
    pub created_at: u64,
    /// Duration of bought token will be locked until claimable
    pub lock_duration: u64,
    /// Duration of bought token will be vested until claimable
    pub vest_duration: u64,
    /// When the lock ends
    pub lock_end_time: u64,
    /// When the vesting ends
    pub vesting_end_time: u64,
    /// Total claimed base token. For statistic purpose only
    pub total_claimed_token: u64,
    /// Maximum deposit fee that can be charged to the buyer
    pub max_deposit_fee: u64,
    pub total_refunded_quote_token: u64,
    /// Deposit fee in basis points (bps). 100 bps = 1%
    pub deposit_fee_bps: u16,
    /// Whitelist mode
    pub whitelist_mode: u8,
    /// Presale mode
    pub presale_mode: u8,
    /// What to do with unsold base token. Only applicable for fixed price presale mode
    pub fixed_price_presale_unlock_unsold_token_action: u8,
    pub padding1: [u8; 11],
    /// Presale rate. Only applicable for fixed price presale mode
    pub fixed_price_presale_q_price: u128,
}

pub struct PresaleInitializeArgs {
    pub tokenomic_params: TokenomicArgs,
    pub presale_params: PresaleArgs,
    pub locked_vesting_params: Option<LockedVestingArgs>,
    pub fixed_price_presale_params: Option<FixedPricePresaleExtraArgs>,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_vault: Pubkey,
    pub quote_token_vault: Pubkey,
    pub owner: Pubkey,
    pub current_timestamp: u64,
}

impl Presale {
    pub fn initialize(&mut self, args: PresaleInitializeArgs) {
        let PresaleInitializeArgs {
            tokenomic_params,
            presale_params,
            locked_vesting_params,
            fixed_price_presale_params,
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
            current_timestamp,
        } = args;

        self.owner = owner;
        self.base_mint = base_mint;
        self.quote_mint = quote_mint;
        self.base_token_vault = base_token_vault;
        self.quote_token_vault = quote_token_vault;

        let TokenomicArgs {
            presale_pool_supply,
            creator_supply,
        } = tokenomic_params;

        self.presale_supply = presale_pool_supply;
        self.creator_supply = creator_supply;

        if let Some(LockedVestingArgs {
            lock_duration,
            vest_duration,
        }) = locked_vesting_params
        {
            self.lock_duration = lock_duration;
            self.vest_duration = vest_duration;
        }

        let PresaleArgs {
            presale_maximum_cap,
            presale_minimum_cap,
            buyer_minimum_deposit_cap,
            buyer_maximum_deposit_cap,
            presale_start_time,
            presale_end_time,
            max_deposit_fee,
            deposit_fee_bps,
            whitelist_mode,
            presale_mode,
        } = presale_params;

        self.presale_maximum_cap = presale_maximum_cap;
        self.presale_minimum_cap = presale_minimum_cap;
        self.buyer_minimum_deposit_cap = buyer_minimum_deposit_cap;
        self.buyer_maximum_deposit_cap = buyer_maximum_deposit_cap;
        self.presale_start_time = presale_start_time;
        self.presale_end_time = presale_end_time;
        self.whitelist_mode = whitelist_mode;
        self.presale_mode = presale_mode;
        self.max_deposit_fee = max_deposit_fee;
        self.deposit_fee_bps = deposit_fee_bps;
        self.created_at = current_timestamp;

        if let Some(FixedPricePresaleExtraArgs {
            unsold_token_action,
            q_price,
            ..
        }) = fixed_price_presale_params
        {
            self.fixed_price_presale_unlock_unsold_token_action = unsold_token_action;
            self.fixed_price_presale_q_price = q_price;
        }
    }

    pub fn get_presale_progress(&self, current_timestamp: u64) -> PresaleProgress {
        if current_timestamp < self.presale_start_time {
            return PresaleProgress::NotStarted;
        } else if current_timestamp < self.presale_end_time {
            return PresaleProgress::Ongoing;
        }

        // TODO: Remove debug
        if self.total_deposit > self.presale_maximum_cap {
            unreachable!(
                "Total deposit {} is greater than presale maximum cap {}",
                self.total_deposit, self.presale_maximum_cap
            );
        }

        if self.total_deposit >= self.presale_minimum_cap {
            PresaleProgress::Completed
        } else {
            PresaleProgress::Failed
        }
    }

    pub fn increase_escrow_count(&mut self) -> Result<()> {
        self.total_escrow = self.total_escrow.checked_add(1).unwrap();
        Ok(())
    }

    pub fn advance_progress_to_completed(&mut self, current_timestamp: u64) -> Result<()> {
        self.presale_end_time = current_timestamp;

        self.lock_end_time = self
            .presale_end_time
            .checked_add(self.lock_duration)
            .unwrap();

        self.vesting_end_time = self.lock_end_time.checked_add(self.vest_duration).unwrap();

        Ok(())
    }

    pub fn get_remaining_deposit_quota(&self) -> Result<u64> {
        let remaining_quota = self
            .presale_maximum_cap
            .checked_sub(self.total_deposit)
            .unwrap();

        Ok(remaining_quota)
    }

    pub fn deposit(
        &mut self,
        escrow: &mut Escrow,
        deposit_fee_included_amount: u64,
        deposit_fee: u64,
    ) -> Result<()> {
        let deposit_fee_excluded_amount = deposit_fee_included_amount
            .checked_sub(deposit_fee)
            .unwrap();

        self.total_deposit = self
            .total_deposit
            .checked_add(deposit_fee_excluded_amount)
            .unwrap();
        self.total_deposit_fee = self.total_deposit_fee.checked_add(deposit_fee).unwrap();

        escrow.deposit(deposit_fee_excluded_amount, deposit_fee)?;

        self.total_escrow = self.total_escrow.checked_add(1).unwrap();

        Ok(())
    }

    pub fn withdraw(&mut self, escrow: &mut Escrow, amount: u64) -> Result<u64> {
        let fee_amount_withdrawn = escrow.withdraw(amount)?;

        self.total_deposit = self.total_deposit.checked_sub(amount).unwrap();
        self.total_deposit_fee = self
            .total_deposit_fee
            .checked_sub(fee_amount_withdrawn)
            .unwrap();

        let total_withdrawn_amount = amount.checked_add(fee_amount_withdrawn).unwrap();
        Ok(total_withdrawn_amount)
    }

    pub fn in_locking_period(&self, current_timestamp: u64) -> bool {
        current_timestamp >= self.presale_end_time && current_timestamp < self.lock_end_time
    }

    pub fn claim(&mut self, escrow: &mut Escrow, amount: u64) -> Result<()> {
        self.total_claimed_token = self.total_claimed_token.checked_add(amount).unwrap();
        escrow.claim(amount)
    }
}
