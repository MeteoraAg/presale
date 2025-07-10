use anchor_lang::prelude::*;

// Supported quote mints
const SOL: Pubkey = Pubkey::from_str_const("So11111111111111111111111111111111111111112");
const USDC: Pubkey = Pubkey::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const QUOTE_MINTS: [Pubkey; 2] = [SOL, USDC];

// Minimum time window for presale
pub const MINIMUM_PRESALE_DURATION: u64 = 60; // 1 minutes

// Maximum time window for presale
pub const MAXIMUM_PRESALE_DURATION: u64 = 60 * 60 * 24 * 30; // 30 days

pub const MAXIMUM_DURATION_UNTIL_PRESALE: u64 = 60 * 60 * 24 * 30; // 30 days

pub const MAXIMUM_LOCK_AND_VEST_DURATION: u64 = 60 * 60 * 24 * 365 * 10; // 10 year

// PDA's seeds
pub mod seeds {
    pub const PRESALE_AUTHORITY_PREFIX: &[u8] = b"presale_authority";
    pub const PRESALE_PREFIX: &[u8] = b"presale";
    pub const BASE_VAULT_PREFIX: &[u8] = b"base_vault";
    pub const QUOTE_VAULT_PREFIX: &[u8] = b"quote_vault";
    pub const FIXED_PRICE_PRESALE_PARAM_PREFIX: &[u8] = b"fixed_price_param";
    pub const ESCROW_PREFIX: &[u8] = b"escrow";
    pub const MERKLE_ROOT_CONFIG_PREFIX: &[u8] = b"merkle_root";
}

pub const MAX_DEPOSIT_FEE_BPS: u64 = 1000; // 10%
pub const PROTOCOL_FEE_BPS: u64 = 2000; // 20%
