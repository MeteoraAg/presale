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

impl WhitelistMode {
    pub fn is_permissioned(&self) -> bool {
        match self {
            WhitelistMode::Permissionless => false,
            WhitelistMode::PermissionWithMerkleProof | WhitelistMode::PermissionWithAuthority => {
                true
            }
        }
    }
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
    /// Presale version
    pub version: u8,
    /// Presale mode
    pub presale_mode: u8,
    /// Whitelist mode
    pub whitelist_mode: u8,
    pub padding0: [u8; 5],
    /// Presale target raised capital
    pub presale_maximum_cap: u64,
    /// Presale minimum raised capital. Else, presale consider as failed.
    pub presale_minimum_cap: u64,
    /// When presale starts
    pub presale_start_time: u64,
    /// When presale ends. Presale can be ended earlier by creator if raised capital is reached (based on presale mode).
    pub presale_end_time: u64,
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
    /// Total deposit fee collected
    pub total_deposit_fee: u64,
    /// Determine whether creator collected the deposit fee
    pub deposit_fee_collected: u8,
    /// Padding
    pub padding1: [u8; 7],
    /// Determine whether creator withdrawn the raised capital
    pub has_creator_withdrawn: u8,
    /// Base token program flag
    pub base_token_program_flag: u8,
    /// Quote token program flag
    pub quote_token_program_flag: u8,
    /// Total presale registry count
    pub total_presale_registry_count: u8,
    /// What to do with unsold base token
    pub unsold_token_action: u8,
    /// Whether the fixed price presale unsold token action has been performed
    pub is_unsold_token_action_performed: u8,
    /// How many % of the token supply is released immediately
    pub immediate_release_bps: u16,
    /// Presale rate. Only applicable for fixed price presale mode
    pub fixed_price_presale_q_price: u128,
    pub padding3: [u128; 6],
    /// Presale registries. Note: Supporting more registries will causes increased account size.
    pub presale_registries: [PresaleRegistry; MAX_PRESALE_REGISTRY_COUNT],
}

static_assertions::const_assert_eq!(Presale::INIT_SPACE, 1264);
static_assertions::assert_eq_align!(Presale, u128);

pub struct PresaleInitializeArgs<'a> {
    pub presale_params: PresaleArgs,
    pub presale_registries: &'a [PresaleRegistryArgs],
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
            presale_params,
            locked_vesting_params,
            fixed_price_presale_params,
            presale_registries,
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

        for (idx, registry) in presale_registries.iter().enumerate() {
            self.presale_registries[idx].init(
                registry.presale_supply,
                registry.buyer_minimum_deposit_cap,
                registry.buyer_maximum_deposit_cap,
                registry.deposit_fee_bps,
            );

            self.presale_supply = self.presale_supply.safe_add(registry.presale_supply)?;
        }

        self.total_presale_registry_count = presale_registries.len() as u8;

        let PresaleArgs {
            presale_maximum_cap,
            presale_minimum_cap,
            presale_end_time,
            whitelist_mode,
            presale_mode,
            ..
        } = presale_params;

        self.presale_maximum_cap = presale_maximum_cap;
        self.presale_minimum_cap = presale_minimum_cap;
        self.presale_start_time =
            presale_params.get_presale_start_time_without_going_backwards(current_timestamp);
        self.presale_end_time = presale_end_time;
        self.whitelist_mode = whitelist_mode;
        self.presale_mode = presale_mode;
        self.created_at = current_timestamp;

        if let Some(LockedVestingArgs {
            lock_duration,
            vest_duration,
            immediately_release_bps,
            ..
        }) = locked_vesting_params
        {
            self.lock_duration = lock_duration;
            self.vest_duration = vest_duration;
            self.immediate_release_bps = immediately_release_bps;

            self.recalculate_presale_timing(self.presale_end_time)?;
        }

        if let Some(FixedPricePresaleExtraArgs {
            unsold_token_action,
            q_price,
            ..
        }) = fixed_price_presale_params
        {
            self.unsold_token_action = unsold_token_action;
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

        if self.total_deposit >= self.presale_minimum_cap {
            PresaleProgress::Completed
        } else {
            PresaleProgress::Failed
        }
    }

    pub fn increase_escrow_count(&mut self, registry_index: u8) -> Result<()> {
        let presale_registry = self.get_presale_registry_mut(registry_index.into())?;
        presale_registry.increase_escrow_count()?;
        self.total_escrow = self.total_escrow.safe_add(1)?;
        Ok(())
    }

    pub fn decrease_escrow_count(&mut self, registry_index: u8) -> Result<()> {
        let presale_registry = self.get_presale_registry_mut(registry_index.into())?;
        presale_registry.decrease_escrow_count()?;
        self.total_escrow = self.total_escrow.safe_sub(1)?;
        Ok(())
    }

    fn recalculate_presale_timing(&mut self, new_presale_end_time: u64) -> Result<()> {
        self.presale_end_time = new_presale_end_time;

        self.lock_start_time = self.presale_end_time.safe_add(1)?;
        self.lock_end_time = self.lock_start_time.safe_add(self.lock_duration)?;

        self.vesting_start_time = self.lock_end_time.safe_add(1)?;
        self.vesting_end_time = self.vesting_start_time.safe_add(self.vest_duration)?;

        Ok(())
    }

    pub fn advance_progress_to_completed(&mut self, current_timestamp: u64) -> Result<()> {
        self.recalculate_presale_timing(current_timestamp)
    }

    pub fn get_remaining_deposit_quota(&self) -> Result<u64> {
        let remaining_quota = self.presale_maximum_cap.safe_sub(self.total_deposit)?;
        Ok(remaining_quota)
    }

    pub fn deposit(
        &mut self,
        escrow: &mut Escrow,
        deposit_amount: u64,
    ) -> Result<DepositFeeIncludedCalculation> {
        let presale_registry = self.get_presale_registry_mut(escrow.registry_index.into())?;

        let deposit_fee_calculation =
            presale_registry.calculate_deposit_fee_included_amount(deposit_amount)?;

        presale_registry.deposit(escrow, deposit_amount, deposit_fee_calculation.fee)?;

        self.total_deposit = self.total_deposit.safe_add(deposit_amount)?;
        self.total_deposit_fee = self
            .total_deposit_fee
            .safe_add(deposit_fee_calculation.fee)?;

        Ok(deposit_fee_calculation)
    }

    pub fn withdraw(&mut self, escrow: &mut Escrow, amount: u64) -> Result<()> {
        let presale_registry = self.get_presale_registry_mut(escrow.registry_index.into())?;
        presale_registry.withdraw(escrow, amount)?;
        self.total_deposit = self.total_deposit.safe_sub(amount)?;
        Ok(())
    }

    pub fn update_total_refunded_quote_token(
        &mut self,
        amount: u64,
        registry_index: u8,
    ) -> Result<()> {
        let presale_registry = self.get_presale_registry_mut(registry_index.into())?;
        presale_registry.update_total_refunded_quote_token(amount)?;
        self.total_refunded_quote_token = self.total_refunded_quote_token.safe_add(amount)?;
        Ok(())
    }

    pub fn is_unsold_price_token_action_performed(&self) -> bool {
        self.is_unsold_token_action_performed != 0
    }

    pub fn set_unsold_token_action_performed(&mut self) -> Result<()> {
        self.is_unsold_token_action_performed = 1;
        Ok(())
    }

    pub fn allow_withdraw_remaining_quote(&self, presale_progress: PresaleProgress) -> bool {
        let presale_mode = PresaleMode::from(self.presale_mode);

        presale_progress == PresaleProgress::Failed
            || (presale_progress == PresaleProgress::Completed
                && presale_mode == PresaleMode::Prorata)
    }

    pub fn get_remaining_quote(&self) -> u64 {
        self.total_deposit.saturating_sub(self.presale_maximum_cap)
    }

    pub fn validate_and_get_escrow_remaining_quote(
        &self,
        escrow: &Escrow,
        current_timestamp: u64,
    ) -> Result<EscrowRemainingQuoteResult> {
        // 1. Ensure presale is in failed or prorata completed state
        let presale_progress = self.get_presale_progress(current_timestamp);
        require!(
            self.allow_withdraw_remaining_quote(presale_progress),
            PresaleError::PresaleNotOpenForWithdrawRemainingQuote
        );

        let (refund_deposit_amount, refund_fee_amount) =
            if presale_progress == PresaleProgress::Failed {
                // 2. Failed presale will refund all tokens to the owner
                (escrow.total_deposit, escrow.total_deposit_fee)
            } else {
                // 3. Prorata (success presale) will refund only the overflow (unused) quote token
                let presale_registry = self.get_presale_registry(escrow.registry_index.into())?;

                let RemainingQuote {
                    refund_amount,
                    refund_fee,
                } = presale_registry
                    .get_remaining_quote(self.get_remaining_quote(), self.total_deposit)?;

                // To be fair to all participants in the registry (same price), refund deposit fee charges on remaining quote amount
                let escrow_refund_fee = if presale_registry.total_deposit_fee > 0 {
                    u128::from(escrow.total_deposit_fee)
                        .safe_mul(refund_fee.into())?
                        .safe_div(presale_registry.total_deposit_fee.into())?
                        .safe_cast()?
                } else {
                    0
                };

                let escrow_refund_amount = if presale_registry.total_deposit > 0 {
                    u128::from(escrow.total_deposit)
                        .safe_mul(refund_amount.into())?
                        .safe_div(presale_registry.total_deposit.into())?
                        .safe_cast()?
                } else {
                    0
                };

                (escrow_refund_amount, escrow_refund_fee)
            };

        Ok(EscrowRemainingQuoteResult {
            refund_deposit_amount,
            refund_fee_amount,
        })
    }

    pub fn get_total_unsold_token(&self, presale_handler: &dyn PresaleModeHandler) -> Result<u64> {
        let total_token_sold = presale_handler.get_total_base_token_sold(self)?;
        let total_token_unsold = self.presale_supply.safe_sub(total_token_sold)?;

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
        let presale_registry = self.get_presale_registry_mut(escrow.registry_index.into())?;
        let claimed_amount = escrow.claim()?;
        presale_registry.update_total_claim_amount(claimed_amount)?;
        self.total_claimed_token = self.total_claimed_token.safe_add(claimed_amount)?;

        Ok(())
    }

    pub fn get_presale_registry(&self, index: usize) -> Result<&PresaleRegistry> {
        self.presale_registries
            .get(index)
            .ok_or(PresaleError::InvalidPresaleRegistryIndex.into())
    }

    pub fn get_presale_registry_mut(&mut self, index: usize) -> Result<&mut PresaleRegistry> {
        self.presale_registries
            .get_mut(index)
            .ok_or(PresaleError::InvalidPresaleRegistryIndex.into())
    }

    pub fn is_deposit_fee_collected(&self) -> bool {
        self.deposit_fee_collected == 1
    }

    pub fn set_deposit_fee_collected(&mut self) {
        self.deposit_fee_collected = 1;
    }

    pub fn get_total_collected_fee(&self) -> Result<u64> {
        let presale_mode = PresaleMode::from(self.presale_mode);
        match presale_mode {
            // In prorata, we need to refund deposit fee of remaining quote to allow fair price for participants in the same registry
            PresaleMode::Prorata => {
                let presale_remaining_quote = self.get_remaining_quote();

                let mut total_fee: u64 = 0;

                for registry in self.presale_registries.iter() {
                    // We can early break because registries are ordered from initialized -> uninitialized
                    if registry.is_uninitialized() {
                        break;
                    }

                    let RemainingQuote { refund_fee, .. } = registry
                        .get_remaining_quote(presale_remaining_quote, self.total_deposit)?;

                    let registry_collected_fee = registry.total_deposit_fee.safe_sub(refund_fee)?;
                    total_fee = total_fee.safe_add(registry_collected_fee)?;
                }

                Ok(total_fee)
            }
            PresaleMode::Fcfs | PresaleMode::FixedPrice => Ok(self.total_deposit_fee),
        }
    }
}

pub struct EscrowRemainingQuoteResult {
    pub refund_deposit_amount: u64,
    pub refund_fee_amount: u64,
}
