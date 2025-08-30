use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct ClosePermissionedServerMetadataCtx<'info> {
    #[account(
        has_one = owner,
    )]
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        mut,
        has_one = presale,
        close = rent_receiver,
    )]
    pub permissioned_server_metadata: Account<'info, PermissionedServerMetadata>,

    /// CHECK: Rent receiver account, which will receive the remaining rent after closing the metadata.
    #[account(mut)]
    pub rent_receiver: UncheckedAccount<'info>,

    pub owner: Signer<'info>,
}

pub fn handle_close_permissioned_server_metadata(
    ctx: Context<ClosePermissionedServerMetadataCtx>,
) -> Result<()> {
    let presale = ctx.accounts.presale.load()?;

    let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;
    let progress = presale.get_presale_progress(current_timestamp);

    require!(
        progress == PresaleProgress::Ongoing || progress == PresaleProgress::NotStarted,
        PresaleError::PresaleEnded
    );

    emit_cpi!(EvtPermissionedServerMetadataClose {
        presale: ctx.accounts.presale.key(),
        permissioned_server_metadata: ctx.accounts.permissioned_server_metadata.key(),
    });

    Ok(())
}
