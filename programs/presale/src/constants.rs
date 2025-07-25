// Minimum time window for presale
pub const MINIMUM_PRESALE_DURATION: u64 = 60; // 1 minutes

// Maximum time window for presale
pub const MAXIMUM_PRESALE_DURATION: u64 = 60 * 60 * 24 * 30; // 30 days

pub const MAXIMUM_DURATION_UNTIL_PRESALE: u64 = 60 * 60 * 24 * 30; // 30 days

pub const MAXIMUM_LOCK_AND_VEST_DURATION: u64 = 60 * 60 * 24 * 365 * 10; // 10 year

pub const SCALE_OFFSET: u32 = 64; // 2^64

// PDA's seeds
pub mod seeds {
    pub const PRESALE_AUTHORITY_PREFIX: &[u8] = b"presale_authority";
    pub const PRESALE_PREFIX: &[u8] = b"presale";
    pub const BASE_VAULT_PREFIX: &[u8] = b"base_vault";
    pub const QUOTE_VAULT_PREFIX: &[u8] = b"quote_vault";
    pub const FIXED_PRICE_PRESALE_PARAM_PREFIX: &[u8] = b"fixed_price_param";
    pub const ESCROW_PREFIX: &[u8] = b"escrow";
    pub const MERKLE_ROOT_CONFIG_PREFIX: &[u8] = b"merkle_root";
    pub const OPERATOR_PREFIX: &[u8] = b"operator";
}
