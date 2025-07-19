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

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum PresaleProgress {
    #[num_enum(default)]
    /// Presale has not started yet
    NotStarted,
    /// Presale is ongoing
    Ongoing,
    /// Presale is ended
    Completed,
    /// Presale is ended but not enough capital raised
    Failed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum TokenProgramFlags {
    /// SPL Token program
    #[num_enum(default)]
    SplToken,
    /// SPL Token 2022 program
    SplToken2022,
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
    /// Base key
    pub base: Pubkey,
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
    /// Total deposited quote token
    pub total_deposit: u64,
    /// Total number of depositors. For statistic purpose only
    pub total_escrow: u64,
    /// When was the presale created
    pub created_at: u64,
    /// Duration of bought token will be locked until claimable
    pub lock_duration: u64,
    /// Duration of bought token will be vested until claimable
    pub vest_duration: u64,
    /// When the lock starts
    pub lock_start_time: u64,
    /// When the lock ends
    pub lock_end_time: u64,
    /// When the vesting starts
    pub vesting_start_time: u64,
    /// When the vesting ends
    pub vesting_end_time: u64,
    /// Total claimed base token. For statistic purpose only
    pub total_claimed_token: u64,
    /// Total refunded quote token. For statistic purpose only
    pub total_refunded_quote_token: u64,
    pub padding0: u64,
    /// Whitelist mode
    pub whitelist_mode: u8,
    /// Presale mode
    pub presale_mode: u8,
    /// Determine whether creator withdrawn the raised capital
    pub has_creator_withdrawn: u8,
    /// Base token program flag
    pub base_token_program_flag: u8,
    /// Quote token program flag
    pub quote_token_program_flag: u8,
    /// What to do with unsold base token. Only applicable for fixed price presale mode
    pub fixed_price_presale_unsold_token_action: u8,
    pub is_fixed_price_presale_unsold_token_action_performed: u8,
    pub padding2: [u8; 17],
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
    pub base: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
}

fn token_program_to_flag(program: Pubkey) -> TokenProgramFlags {
    if program == anchor_spl::token::ID {
        TokenProgramFlags::SplToken
    } else if program == anchor_spl::token_2022::ID {
        TokenProgramFlags::SplToken2022
    } else {
        unreachable!("Unsupported token program: {}", program);
    }
}

impl Presale {
    pub fn initialize(&mut self, args: PresaleInitializeArgs) -> Result<()> {
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
            base,
            base_token_program,
            quote_token_program,
        } = args;

        self.owner = owner;
        self.base_mint = base_mint;
        self.quote_mint = quote_mint;
        self.base_token_vault = base_token_vault;
        self.quote_token_vault = quote_token_vault;
        self.base = base;
        self.base_token_program_flag = token_program_to_flag(base_token_program).into();
        self.quote_token_program_flag = token_program_to_flag(quote_token_program).into();

        let TokenomicArgs {
            presale_pool_supply,
        } = tokenomic_params;

        self.presale_supply = presale_pool_supply;

        let PresaleArgs {
            presale_maximum_cap,
            presale_minimum_cap,
            buyer_minimum_deposit_cap,
            buyer_maximum_deposit_cap,
            presale_start_time,
            presale_end_time,
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
        self.created_at = current_timestamp;

        if let Some(LockedVestingArgs {
            lock_duration,
            vest_duration,
        }) = locked_vesting_params
        {
            self.lock_duration = lock_duration;
            self.vest_duration = vest_duration;

            self.recalculate_presale_timing(self.presale_end_time)?;
        }

        if let Some(FixedPricePresaleExtraArgs {
            unsold_token_action,
            q_price,
            ..
        }) = fixed_price_presale_params
        {
            self.fixed_price_presale_unsold_token_action = unsold_token_action;
            self.fixed_price_presale_q_price = q_price;
        }

        Ok(())
    }

    pub fn get_presale_progress(&self, current_timestamp: u64) -> PresaleProgress {
        if current_timestamp < self.presale_start_time {
            return PresaleProgress::NotStarted;
        } else if current_timestamp <= self.presale_end_time {
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

    pub fn decrease_escrow_count(&mut self) -> Result<()> {
        self.total_escrow = self.total_escrow.checked_sub(1).unwrap();
        Ok(())
    }

    fn recalculate_presale_timing(&mut self, new_presale_end_time: u64) -> Result<()> {
        self.presale_end_time = new_presale_end_time;

        self.lock_start_time = self.presale_end_time.checked_add(1).unwrap();
        self.lock_end_time = self
            .lock_start_time
            .checked_add(self.lock_duration)
            .unwrap();

        self.vesting_start_time = self.lock_end_time.checked_add(1).unwrap();
        self.vesting_end_time = self.lock_end_time.checked_add(self.vest_duration).unwrap();

        Ok(())
    }

    pub fn advance_progress_to_completed(&mut self, current_timestamp: u64) -> Result<()> {
        self.recalculate_presale_timing(current_timestamp)
    }

    pub fn get_remaining_deposit_quota(&self) -> Result<u64> {
        let remaining_quota = self
            .presale_maximum_cap
            .checked_sub(self.total_deposit)
            .unwrap();

        Ok(remaining_quota)
    }

    pub fn deposit(&mut self, escrow: &mut Escrow, deposit_amount: u64) -> Result<()> {
        self.total_deposit = self.total_deposit.checked_add(deposit_amount).unwrap();
        escrow.deposit(deposit_amount)?;
        Ok(())
    }

    pub fn withdraw(&mut self, escrow: &mut Escrow, amount: u64) -> Result<()> {
        escrow.withdraw(amount)?;
        self.total_deposit = self.total_deposit.checked_sub(amount).unwrap();
        Ok(())
    }

    pub fn in_locking_period(&self, current_timestamp: u64) -> bool {
        current_timestamp >= self.lock_start_time && current_timestamp <= self.lock_end_time
    }

    pub fn update_total_refunded_quote_token(&mut self, amount: u64) -> Result<()> {
        self.total_refunded_quote_token =
            self.total_refunded_quote_token.checked_add(amount).unwrap();

        Ok(())
    }

    pub fn is_fixed_price_presale_unsold_token_action_performed(&self) -> bool {
        self.is_fixed_price_presale_unsold_token_action_performed != 0
    }

    pub fn set_fixed_price_presale_unsold_token_action_performed(&mut self) -> Result<()> {
        self.is_fixed_price_presale_unsold_token_action_performed = 1;
        Ok(())
    }

    pub fn allow_withdraw_remaining_quote(&self, presale_progress: PresaleProgress) -> bool {
        let presale_mode = PresaleMode::from(self.presale_mode);

        presale_progress == PresaleProgress::Failed
            || (presale_progress == PresaleProgress::Completed
                && presale_mode == PresaleMode::Prorata)
    }

    pub fn validate_and_get_escrow_remaining_quote(
        &self,
        escrow: &Escrow,
        current_timestamp: u64,
    ) -> Result<u64> {
        // 1. Ensure presale is in failed or prorata completed state
        let presale_progress = self.get_presale_progress(current_timestamp);
        require!(
            self.allow_withdraw_remaining_quote(presale_progress),
            PresaleError::PresaleNotOpenForWithdrawRemainingQuote
        );

        let refund_amount = if presale_progress == PresaleProgress::Failed {
            // 2. Failed presale will refund all tokens to the owner
            escrow.total_deposit
        } else {
            // 3. Prorata will refund only the overflow (unused) quote token
            let remaining_quote_amount =
                self.total_deposit.saturating_sub(self.presale_maximum_cap);

            u128::from(escrow.total_deposit)
                .checked_mul(remaining_quote_amount.into())
                .unwrap()
                .checked_div(self.total_deposit.into())
                .unwrap()
                .try_into()
                .unwrap()
        };

        Ok(refund_amount)
    }

    pub fn get_total_unsold_token(&self, presale_handler: &dyn PresaleModeHandler) -> Result<u64> {
        let total_token_sold = presale_handler.get_total_base_token_sold(self)?;
        let total_token_unsold = self.presale_supply.checked_sub(total_token_sold).unwrap();

        Ok(total_token_unsold)
    }

    pub fn has_creator_withdrawn(&self) -> bool {
        self.has_creator_withdrawn != 0
    }

    pub fn update_creator_withdrawn(&mut self) -> Result<()> {
        self.has_creator_withdrawn = 1;
        Ok(())
    }

    pub fn claim(&mut self, escrow: &mut Escrow) -> Result<()> {
        self.total_claimed_token = self
            .total_claimed_token
            .checked_add(escrow.pending_claim_token)
            .unwrap();

        escrow.claim()
    }
}
