use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializePresaleArgs {
    pub token_info: TokenInfoArgs,
    pub tokenomic: TokenomicArgs,
    pub presale_params: PresaleArgs,
    pub locked_vesting_params: Option<LockedVestingArgs>,
}
impl InitializePresaleArgs {
    pub fn validate(&self) -> Result<()> {
        self.token_info.validate()?;
        self.tokenomic.validate()?;

        let current_timestamp = Clock::get()?.unix_timestamp as u64;
        self.presale_params.validate(current_timestamp)?;

        if let Some(locked_vesting) = &self.locked_vesting_params {
            locked_vesting.validate()?;
        }
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TokenInfoArgs {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

impl TokenInfoArgs {
    pub fn validate(&self) -> Result<()> {
        // When it's 12 decimal place, max supply is 18_446_744.073_709_551_615. ~18k
        require!(
            self.decimals > 0 && self.decimals <= 12,
            PresaleError::InvalidTokenInfo
        );
        require!(self.name.len() > 0, PresaleError::InvalidTokenInfo);
        require!(self.symbol.len() > 0, PresaleError::InvalidTokenInfo);
        require!(self.uri.len() > 0, PresaleError::InvalidTokenInfo);

        Ok(())
    }
}

// presale_pool_supply + creator_supply = supply
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct TokenomicArgs {
    pub presale_pool_supply: u64,
    pub creator_supply: u64,
}

impl TokenomicArgs {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.presale_pool_supply > 0,
            PresaleError::InvalidTokenSupply
        );
        let total_supply = u128::from(self.presale_pool_supply)
            .checked_add(self.creator_supply.into())
            .unwrap();
        require!(
            total_supply <= u128::from(u64::MAX),
            PresaleError::InvalidTokenSupply
        );
        Ok(())
    }

    pub fn get_total_supply(&self) -> Result<u64> {
        Ok(self
            .presale_pool_supply
            .checked_add(self.creator_supply)
            .unwrap())
    }
}

/// Presale parameters
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone)]
pub struct PresaleArgs {
    pub presale_maximum_cap: u64,
    pub presale_minimum_cap: u64,
    pub buyer_minimum_deposit_cap: u64,
    pub buyer_maximum_deposit_cap: u64,
    pub presale_start_time: u64,
    pub presale_end_time: u64,
    pub max_deposit_fee: u64,
    pub deposit_fee_bps: u16,
    pub whitelist_mode: u8,
    pub presale_mode: u8,
}

impl PresaleArgs {
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
            self.buyer_minimum_deposit_cap > 0,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            self.presale_start_time >= current_timestamp,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            self.presale_end_time > self.presale_start_time,
            PresaleError::InvalidPresaleInfo
        );

        let presale_duration = self
            .presale_end_time
            .checked_sub(self.presale_start_time)
            .unwrap();

        require!(
            presale_duration >= MINIMUM_PRESALE_DURATION
                && presale_duration <= MAXIMUM_PRESALE_DURATION,
            PresaleError::InvalidPresaleInfo
        );

        let duration_until_presale = current_timestamp
            .checked_sub(self.presale_start_time)
            .unwrap();

        require!(
            duration_until_presale <= MAXIMUM_DURATION_UNTIL_PRESALE,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            u64::from(self.deposit_fee_bps) <= MAX_DEPOSIT_FEE_BPS,
            PresaleError::InvalidPresaleInfo
        );

        if self.deposit_fee_bps > 0 {
            require!(self.max_deposit_fee > 0, PresaleError::InvalidPresaleInfo);
        }

        WhitelistMode::try_from(self.whitelist_mode).unwrap();
        PresaleMode::try_from(self.presale_mode).unwrap();

        Ok(())
    }
}

/// Vest user bought token
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub struct LockedVestingArgs {
    /// Lock duration until buyer can claim the token
    pub lock_duration: u64,
    /// Vesting duration until buyer can claim the token
    pub vest_duration: u64,
}

impl LockedVestingArgs {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.vest_duration >= self.lock_duration,
            PresaleError::InvalidLockVestingInfo
        );

        let total_duration = self.vest_duration.checked_add(self.lock_duration).unwrap();
        require!(
            total_duration < MAXIMUM_LOCK_AND_VEST_DURATION,
            PresaleError::InvalidLockVestingInfo
        );

        Ok(())
    }
}
