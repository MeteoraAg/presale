// Minimum time window for presale
pub const MINIMUM_PRESALE_DURATION: u64 = 60; // 1 minutes

// Maximum time window for presale
pub const MAXIMUM_PRESALE_DURATION: u64 = 60 * 60 * 24 * 30; // 30 days

pub const MAXIMUM_DURATION_UNTIL_PRESALE: u64 = 60 * 60 * 24 * 30; // 30 days

pub const MAXIMUM_LOCK_AND_VEST_DURATION: u64 = 60 * 60 * 24 * 365 * 10; // 10 year

pub const SCALE_OFFSET: u32 = 64; // 2^64

pub const SCALE_MULTIPLIER: u128 = 1u128 << SCALE_OFFSET; // 2^64

pub const MAX_PRESALE_REGISTRY_COUNT: usize = 5;

pub const MAX_DEPOSIT_FEE_BPS: u16 = 5000; // 50%

// Only permissioned whitelist mode allowed to have multiple presale registries. The constant defined below is the default index for permissionless registries.
pub const DEFAULT_PERMISSIONLESS_REGISTRY_INDEX: u8 = 0;

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
    pub const PERMISSIONED_SERVER_METADATA_PREFIX: &[u8] = b"server_metadata";
}
