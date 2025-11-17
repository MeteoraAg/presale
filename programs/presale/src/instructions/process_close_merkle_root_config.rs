use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CloseMerkleRootConfigCtx<'info> {
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        mut,
        has_one = presale,
        close = rent_receiver
    )]
    pub merkle_root_config: AccountLoader<'info, MerkleRootConfig>,

    /// CHECK: Rent receiver
    #[account(mut)]
    pub rent_receiver: UncheckedAccount<'info>,

    #[account(
        constraint = presale.load()?.owner == creator.key() @ PresaleError::InvalidCreatorAccount
    )]
    pub creator: Signer<'info>,
}

pub fn handle_close_merkle_root_config(ctx: Context<CloseMerkleRootConfigCtx>) -> Result<()> {
    let presale = ctx.accounts.presale.load()?;

    let current_timestamp = Clock::get()?.unix_timestamp.safe_cast()?;
    let progress = presale.get_presale_progress(current_timestamp);

    // Allow presale close when
    // 1. Presale not started (mistake correction)
    // 2. Presale ended (save rent)
    // 3. Presale failed (save rent)
    require!(
        progress != PresaleProgress::Ongoing,
        PresaleError::PresaleOngoing
    );

    emit_cpi!(EvtCloseMerkleRootConfig {
        presale: ctx.accounts.presale.key(),
        owner: ctx.accounts.creator.key(),
        merkle_root_config: ctx.accounts.merkle_root_config.key(),
    });

    Ok(())
}
