use crate::{
    instructions::create_escrow::process_create_escrow::{
        process_create_escrow, HandleCreateEscrowArgs,
    },
    *,
};
use anchor_lang::solana_program::hash::hashv;

// We need to discern between leaf and intermediate nodes to prevent trivial second
// pre-image attacks.
// https://flawed.net.nz/2018/02/21/attacking-merkle-trees-with-a-second-preimage-attack
const LEAF_PREFIX: &[u8] = &[0];

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct CreatePermissionedEscrowWithMerkleProofParams {
    pub proof: Vec<[u8; 32]>,
    pub registry_index: u8,
    pub deposit_cap: u64,
    pub padding: [u8; 32],
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: CreatePermissionedEscrowWithMerkleProofParams)]
pub struct CreatePermissionedEscrowWithMerkleProofCtx<'info> {
    #[account(mut)]
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::ESCROW_PREFIX,
            presale.key().as_ref(),
            owner.key().as_ref(),
            params.registry_index.to_le_bytes().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + Escrow::INIT_SPACE
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    /// CHECK: Owner of the escrow account
    pub owner: UncheckedAccount<'info>,

    #[account(has_one = presale)]
    pub merkle_root_config: AccountLoader<'info, MerkleRootConfig>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_permissioned_escrow_with_merkle_proof(
    ctx: Context<CreatePermissionedEscrowWithMerkleProofCtx>,
    params: CreatePermissionedEscrowWithMerkleProofParams,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;

    // 1. Ensure presale is permissioned with merkle proof
    let whitelist_mode: WhitelistMode = presale.whitelist_mode.safe_cast()?;
    require!(
        whitelist_mode == WhitelistMode::PermissionWithMerkleProof,
        PresaleError::InvalidPresaleWhitelistMode
    );

    let CreatePermissionedEscrowWithMerkleProofParams {
        registry_index,
        proof,
        deposit_cap,
        ..
    } = params;

    // 2. Verify the merkle proof
    let merkle_root_config = ctx.accounts.merkle_root_config.load()?;
    let node = hashv(&[
        &ctx.accounts.owner.key().to_bytes(),
        registry_index.to_le_bytes().as_ref(),
        deposit_cap.to_le_bytes().as_ref(),
    ]);
    let node = hashv(&[LEAF_PREFIX, &node.to_bytes()]);
    require!(
        verify(proof, merkle_root_config.root, node.to_bytes()),
        PresaleError::InvalidMerkleProof
    );

    process_create_escrow(HandleCreateEscrowArgs {
        presale: &mut presale,
        escrow: &ctx.accounts.escrow,
        presale_pubkey: ctx.accounts.presale.key(),
        owner_pubkey: ctx.accounts.owner.key(),
        registry_index,
        deposit_cap: Some(deposit_cap),
    })?;

    emit_cpi!(EvtEscrowCreate {
        presale: ctx.accounts.presale.key(),
        owner: ctx.accounts.owner.key(),
        whitelist_mode: presale.whitelist_mode,
        total_escrow_count: presale.total_escrow,
    });

    Ok(())
}

/// Modified version of https://github.com/saber-hq/merkle-distributor/blob/ac937d1901033ecb7fa3b0db22f7b39569c8e052/programs/merkle-distributor/src/merkle_proof.rs#L8
/// This function deals with verification of Merkle trees (hash trees).
/// Direct port of https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v3.4.0/contracts/cryptography/MerkleProof.sol
/// Returns true if a `leaf` can be proved to be a part of a Merkle tree
/// defined by `root`. For this, a `proof` must be provided, containing
/// sibling hashes on the branch from the leaf to the root of the tree. Each
/// pair of leaves and each pair of pre-images are assumed to be sorted.
pub fn verify(proof: Vec<[u8; 32]>, root: [u8; 32], leaf: [u8; 32]) -> bool {
    let mut computed_hash = leaf;
    for proof_element in proof.into_iter() {
        if computed_hash <= proof_element {
            // Hash(current computed hash + current element of the proof)
            computed_hash = hashv(&[&[1u8], &computed_hash, &proof_element]).to_bytes();
        } else {
            // Hash(current element of the proof + current computed hash)
            computed_hash = hashv(&[&[1u8], &proof_element, &computed_hash]).to_bytes();
        }
    }
    // Check if the computed hash (root) is equal to the provided root
    computed_hash == root
}
