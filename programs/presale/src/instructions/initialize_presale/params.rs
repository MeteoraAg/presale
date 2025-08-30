use crate::*;

fn validate_presale_registries(
    presale_registries: &[PresaleRegistryArgs],
    presale_params: &PresaleArgs,
) -> Result<()> {
    require!(
        presale_registries.len() > 0 && presale_registries.len() <= MAX_PRESALE_REGISTRY_COUNT,
        PresaleError::InvalidPresaleInfo
    );

    let mut presale_supply = 0u128;

    for registry in presale_registries {
        registry.validate(presale_params)?;
        presale_supply = presale_supply.safe_add(u128::from(registry.presale_supply))?;
    }

    require!(
        presale_supply <= u128::from(u64::MAX),
        PresaleError::InvalidTokenSupply
    );

    // Must have at least 1 presale registry
    require!(presale_supply > 0, PresaleError::InvalidTokenSupply);

    // If presale have multiple registries. Whitelist mode must be Permissioned mode.
    // Reason: It make no sense for a single user to deposit to multiple registries which might have different price when it's dynamic price mode.
    // Note: Presale creator have to make sure user doesn't duplicate across registries.
    if presale_registries.len() > 1 {
        let whitelist_mode = WhitelistMode::from(presale_params.whitelist_mode);
        require!(
            whitelist_mode.is_permissioned(),
            PresaleError::MultiplePresaleRegistriesNotAllowed
        );
    }

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Default)]
pub struct InitializePresaleArgs {
    pub presale_params: PresaleArgs,
    pub locked_vesting_params: OptionalNonZeroLockedVestingArgs,
    pub padding: [u8; 32],
    pub presale_registries: Vec<PresaleRegistryArgs>,
}

impl InitializePresaleArgs {
    pub fn validate(&self) -> Result<()> {
        let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;
        self.presale_params.validate(current_timestamp)?;

        validate_presale_registries(&self.presale_registries, &self.presale_params)?;

        let locked_vesting_params: Option<LockedVestingArgs> = self.locked_vesting_params.into();

        if let Some(locked_vesting) = locked_vesting_params {
            locked_vesting.validate()?;
        }

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Default, Debug)]
pub struct PresaleRegistryArgs {
    pub buyer_minimum_deposit_cap: u64,
    pub buyer_maximum_deposit_cap: u64,
    pub presale_supply: u64,
    pub deposit_fee_bps: u16,
    pub padding: [u8; 32],
}

impl PresaleRegistryArgs {
    pub fn is_uninitialized(&self) -> bool {
        self.buyer_minimum_deposit_cap == 0
            && self.buyer_maximum_deposit_cap == 0
            && self.presale_supply == 0
            && self.deposit_fee_bps == 0
    }

    pub fn validate(&self, presale_args: &PresaleArgs) -> Result<()> {
        require!(
            self.buyer_maximum_deposit_cap >= self.buyer_minimum_deposit_cap,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            self.buyer_maximum_deposit_cap > 0,
            PresaleError::InvalidPresaleInfo
        );

        require!(
            self.buyer_maximum_deposit_cap <= presale_args.presale_maximum_cap,
            PresaleError::InvalidPresaleInfo
        );

        require!(self.presale_supply > 0, PresaleError::InvalidTokenSupply);

        require!(
            self.deposit_fee_bps <= MAX_DEPOSIT_FEE_BPS,
            PresaleError::InvalidPresaleInfo
        );

        Ok(())
    }
}

/// Presale parameters
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, Default)]
pub struct PresaleArgs {
    pub presale_maximum_cap: u64,
    pub presale_minimum_cap: u64,
    pub presale_start_time: u64,
    pub presale_end_time: u64,
    pub whitelist_mode: u8,
    pub presale_mode: u8,
    pub padding: [u8; 32],
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
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LockedVestingArgs {
    /// Lock duration until buyer can claim the token
    pub lock_duration: u64,
    /// Vesting duration until buyer can claim the token
    pub vest_duration: u64,
    pub padding: [u8; 32],
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

pub type OptionalNonZeroLockedVestingArgs = LockedVestingArgs;

impl TryFrom<Option<LockedVestingArgs>> for OptionalNonZeroLockedVestingArgs {
    type Error = anchor_lang::error::Error;
    fn try_from(args: Option<LockedVestingArgs>) -> std::result::Result<Self, Self::Error> {
        match args {
            None => Ok(Self::default()),
            Some(locked_vesting_args) => {
                if locked_vesting_args == Self::default() {
                    Err(PresaleError::InvalidLockVestingInfo.into())
                } else {
                    Ok(locked_vesting_args)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests to ensure no breaking change on ix data deserialize
    use super::*;

    #[test]
    fn test_ensure_locked_vesting_args_size() {
        let args = LockedVestingArgs::default();
        assert_eq!(args.try_to_vec().unwrap().len(), 48);
    }

    #[test]
    fn test_ensure_initialize_presale_args_size() {
        let args = InitializePresaleArgs::default();
        // The size is based on 0 registry
        assert_eq!(args.try_to_vec().unwrap().len(), 150);
    }

    #[test]
    fn test_ensure_presale_args_size() {
        let args = PresaleArgs::default();
        assert_eq!(args.try_to_vec().unwrap().len(), 66);
    }

    #[test]
    fn test_ensure_presale_registry_args_size() {
        let args = PresaleRegistryArgs::default();
        assert_eq!(args.try_to_vec().unwrap().len(), 58);
    }
}
