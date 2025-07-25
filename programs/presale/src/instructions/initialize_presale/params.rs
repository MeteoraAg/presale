use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Default)]
pub struct InitializePresaleArgs {
    pub tokenomic: TokenomicArgs,
    pub presale_params: PresaleArgs,
    pub locked_vesting_params: Option<LockedVestingArgs>,
    pub padding: [u64; 4],
}

impl InitializePresaleArgs {
    pub fn validate(&self) -> Result<()> {
        self.tokenomic.validate()?;

        let current_timestamp = Clock::get()?.unix_timestamp as u64;
        self.presale_params.validate(current_timestamp)?;

        if let Some(locked_vesting) = &self.locked_vesting_params {
            locked_vesting.validate()?;
        }
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Default)]
pub struct TokenomicArgs {
    pub presale_pool_supply: u64,
    pub padding: [u64; 4],
}

impl TokenomicArgs {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.presale_pool_supply > 0,
            PresaleError::InvalidTokenSupply
        );

        Ok(())
    }
}

/// Presale parameters
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Default)]
pub struct PresaleArgs {
    pub presale_maximum_cap: u64,
    pub presale_minimum_cap: u64,
    pub buyer_minimum_deposit_cap: u64,
    pub buyer_maximum_deposit_cap: u64,
    pub presale_start_time: u64,
    pub presale_end_time: u64,
    pub whitelist_mode: u8,
    pub presale_mode: u8,
    pub padding: [u64; 4],
}

impl PresaleArgs {
    pub fn get_presale_start_time_without_going_backwards(&self, current_timestamp: u64) -> u64 {
        self.presale_start_time.max(current_timestamp)
    }

    pub fn validate(&self, current_timestamp: u64) -> Result<()> {
        require!(
            self.presale_maximum_cap >= self.presale_minimum_cap,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            self.presale_minimum_cap > 0,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            self.buyer_maximum_deposit_cap >= self.buyer_minimum_deposit_cap,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            self.buyer_maximum_deposit_cap > 0,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            self.buyer_maximum_deposit_cap <= self.presale_maximum_cap,
            PresaleError::InvalidPresaleInfo
        );

        let presale_start_time =
            self.get_presale_start_time_without_going_backwards(current_timestamp);

        require!(
            self.presale_end_time > presale_start_time,
            PresaleError::InvalidPresaleInfo
        );

        let presale_duration = self.presale_end_time.safe_sub(presale_start_time)?;

        require!(
            presale_duration >= MINIMUM_PRESALE_DURATION
                && presale_duration <= MAXIMUM_PRESALE_DURATION,
            PresaleError::InvalidPresaleInfo
        );

        let duration_until_presale = presale_start_time.safe_sub(current_timestamp)?;

        require!(
            duration_until_presale <= MAXIMUM_DURATION_UNTIL_PRESALE,
            PresaleError::InvalidPresaleInfo
        );

        let maybe_whitelist_mode = WhitelistMode::try_from(self.whitelist_mode);
        require!(
            maybe_whitelist_mode.is_ok(),
            PresaleError::InvalidPresaleInfo
        );
        let maybe_presale_mode = PresaleMode::try_from(self.presale_mode);
        require!(maybe_presale_mode.is_ok(), PresaleError::InvalidPresaleInfo);

        Ok(())
    }
}

/// Vest user bought token
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub struct LockedVestingArgs {
    /// Lock duration until buyer can claim the token
    pub lock_duration: u64,
    /// Vesting duration until buyer can claim the token
    pub vest_duration: u64,
    pub padding: [u64; 4],
}

impl LockedVestingArgs {
    pub fn validate(&self) -> Result<()> {
        let total_duration = self.vest_duration.safe_add(self.lock_duration)?;
        require!(
            total_duration < MAXIMUM_LOCK_AND_VEST_DURATION,
            PresaleError::InvalidLockVestingInfo
        );

        Ok(())
    }
}
