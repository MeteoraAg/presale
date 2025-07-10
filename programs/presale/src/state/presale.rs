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
    /// When the lock starts
    pub lock_start_time: u64,
    /// When the vesting starts
    pub vesting_start_time: u64,
    /// Maximum deposit fee that can be charged to the buyer
    pub max_deposit_fee: u64,
    /// Deposit fee in basis points (bps). 100 bps = 1%
    pub deposit_fee_bps: u16,
    /// Whitelist mode
    pub whitelist_mode: u8,
    /// Presale mode
    pub presale_mode: u8,
    pub padding0: [u8; 3],
    /// What to do with unsold base token. Only applicable for fixed price presale mode
    pub fixed_price_presale_unlock_unsold_token: u8,
    /// Presale rate. Only applicable for fixed price presale mode
    pub fixed_price_presale_q_price: u128,
}

impl Presale {
    pub fn initialize(
        &mut self,
        tokenomic_params: TokenomicArgs,
        presale_params: PresaleArgs,
        locked_vesting_params: Option<LockedVestingArgs>,
        fixed_price_presale_params: Option<FixedPricePresaleExtraArgs>,
        current_timestamp: u64,
    ) {
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
            self.fixed_price_presale_unlock_unsold_token = unsold_token_action;
            self.fixed_price_presale_q_price = q_price;
        }
    }
}
