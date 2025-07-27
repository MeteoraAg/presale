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
    pub owner: Pubkey,
    pub fixed_price_presale_args: Pubkey,
}

#[event]
pub struct EvtPresaleVaultCreate {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub buyer_maximum_deposit_cap: u64,
    pub buyer_minimum_deposit_cap: u64,
    pub lock_duration: u64,
    pub vest_duration: u64,
    pub whitelist_mode: u8,
    pub presale_mode: u8,
    pub presale_start_time: u64,
    pub presale_end_time: u64,
    pub presale_maximum_cap: u64,
    pub presale_minimum_cap: u64,
}

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
    pub escrow_total_deposit_amount: u64,
    pub presale_total_deposit_amount: u64,
    pub owner: Pubkey,
}

#[event]
pub struct EvtWithdraw {
    pub presale: Pubkey,
    pub escrow: Pubkey,
    pub withdraw_amount: u64,
    pub escrow_total_deposit_amount: u64,
    pub presale_total_deposit_amount: u64,
    pub owner: Pubkey,
}

#[event]
pub struct EvtClaim {
    pub presale: Pubkey,
    pub escrow: Pubkey,
    pub claim_amount: u64,
    pub escrow_total_claim_amount: u64,
    pub presale_total_claim_amount: u64,
    pub owner: Pubkey,
}

#[event]
pub struct EvtWithdrawRemainingQuote {
    pub presale: Pubkey,
    pub escrow: Pubkey,
    pub owner: Pubkey,
    pub amount_refunded: u64,
    pub presale_total_refunded_quote_token: u64,
}

#[event]
pub struct EvtPerformUnsoldBaseTokenAction {
    pub presale: Pubkey,
    pub total_token_unsold: u64,
    pub unsold_base_token_action: u8,
}

#[event]
pub struct EvtEscrowClose {
    pub presale: Pubkey,
    pub escrow: Pubkey,
    pub owner: Pubkey,
    pub rent_receiver: Pubkey,
}

#[event]
pub struct EvtCreatorWithdraw {
    pub presale: Pubkey,
    pub amount: u64,
    pub creator: Pubkey,
    pub presale_progress: u8,
}

#[event]
pub struct EvtEscrowRefresh {
    pub presale: Pubkey,
    pub escrow: Pubkey,
    pub owner: Pubkey,
    pub current_timestamp: u64,
    pub pending_claim_token: u64,
}

#[event]
pub struct EvtOperatorCreate {
    pub creator: Pubkey,
    pub operator: Pubkey,
    pub operator_owner: Pubkey,
}

#[event]
pub struct EvtMerkleProofMetadataCreate {
    pub presale: Pubkey,
    pub merkle_proof_metadata: Pubkey,
    pub proof_url: String,
}

#[event]
pub struct EvtMerkleProofMetadataClose {
    pub presale: Pubkey,
    pub merkle_proof_metadata: Pubkey,
}
