use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CloseMerkleProofMetadataCtx<'info> {
    #[account(
        has_one = owner,
    )]
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        mut,
        has_one = presale,
        close = rent_receiver,
    )]
    pub merkle_proof_metadata: Account<'info, MerkleProofMetadata>,

    /// CHECK: Rent receiver account, which will receive the remaining rent after closing the metadata.
    #[account(mut)]
    pub rent_receiver: UncheckedAccount<'info>,

    pub owner: Signer<'info>,
}

pub fn handle_close_merkle_proof_metadata(ctx: Context<CloseMerkleProofMetadataCtx>) -> Result<()> {
    let presale = ctx.accounts.presale.load()?;

    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let progress = presale.get_presale_progress(current_timestamp);

    require!(
        progress == PresaleProgress::Ongoing || progress == PresaleProgress::NotStarted,
        PresaleError::PresaleEnded
    );

    emit_cpi!(EvtMerkleProofMetadataClose {
        presale: ctx.accounts.presale.key(),
        merkle_proof_metadata: ctx.accounts.merkle_proof_metadata.key(),
    });

    Ok(())
}
