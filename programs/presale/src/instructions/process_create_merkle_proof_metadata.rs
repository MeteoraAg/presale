use crate::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(proof_url: String)]
pub struct CreateMerkleProofMetadataCtx<'info> {
    #[account(
        has_one = owner,
    )]
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::MERKLE_PROOF_METADATA_PREFIX.as_ref(),
            presale.key().as_ref(),
        ],
        payer = owner,
        bump,
        space = 8 + MerkleProofMetadata::space(proof_url)
    )]
    pub merkle_proof_metadata: Account<'info, MerkleProofMetadata>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_merkle_proof_metadata(
    ctx: Context<CreateMerkleProofMetadataCtx>,
    proof_url: String,
) -> Result<()> {
    let presale = ctx.accounts.presale.load()?;

    let whitelist_mode = WhitelistMode::from(presale.whitelist_mode);
    require!(
        whitelist_mode == WhitelistMode::PermissionWithMerkleProof,
        PresaleError::InvalidPresaleWhitelistMode
    );

    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let progress = presale.get_presale_progress(current_timestamp);

    require!(
        progress == PresaleProgress::Ongoing || progress == PresaleProgress::NotStarted,
        PresaleError::PresaleEnded
    );

    ctx.accounts
        .merkle_proof_metadata
        .initialize(ctx.accounts.presale.key(), proof_url.clone())?;

    emit_cpi!(EvtMerkleProofMetadataCreate {
        presale: ctx.accounts.presale.key(),
        merkle_proof_metadata: ctx.accounts.merkle_proof_metadata.key(),
        proof_url
    });

    Ok(())
}
