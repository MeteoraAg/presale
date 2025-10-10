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
    // We don't check for presale progress since
    // 1. In NotStarted / Ongoing state, the owner can close the metadata for any correction.
    // 2. In Completed / Failed state, the metadata is not needed anymore, so closing it is fine.
    // This covered all presale progress states.

    emit_cpi!(EvtPermissionedServerMetadataClose {
        presale: ctx.accounts.presale.key(),
        permissioned_server_metadata: ctx.accounts.permissioned_server_metadata.key(),
    });

    Ok(())
}
