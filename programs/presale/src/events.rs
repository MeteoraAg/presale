use crate::*;

#[event]
pub struct EvtFixedPricePresaleArgsCreate {
    pub presale: Pubkey,
    pub unsold_token_action: u8,
    pub q_price: u128,
}

#[event]
pub struct EvtFixedPricePresaleArgsClose {
    pub presale: Pubkey,
}

#[event]
pub struct EvtPresaleVaultCreate {}

#[event]
pub struct EvtEscrowCreate {
    pub presale: Pubkey,
    pub owner: Pubkey,
    pub whitelist_mode: u8,
    pub total_escrow_count: u64,
}

#[event]
pub struct EvtMerkleRootConfigCreate {
    pub owner: Pubkey,
    pub config: Pubkey,
    pub presale: Pubkey,
    pub version: u64,
    pub root: [u8; 32],
}

#[event]
pub struct EvtDeposit {
    pub presale: Pubkey,
    pub escrow: Pubkey,
    pub deposit_amount: u64,
    pub deposit_fee: u64,
    pub escrow_total_deposit_amount: u64,
    pub escrow_total_deposit_fee: u64,
    pub presale_total_deposit_amount: u64,
    pub presale_total_deposit_fee: u64,
    pub owner: Pubkey,
}
