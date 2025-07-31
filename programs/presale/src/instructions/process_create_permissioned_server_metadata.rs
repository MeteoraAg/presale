use crate::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(proof_url: String)]
pub struct CreatePermissionedServerMetadataCtx<'info> {
    #[account(
        has_one = owner,
    )]
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::PERMISSIONED_SERVER_METADATA_PREFIX.as_ref(),
            presale.key().as_ref(),
        ],
        payer = owner,
        bump,
        space = 8 + PermissionedServerMetadata::space(proof_url)
    )]
    pub permissioned_server_metadata: Account<'info, PermissionedServerMetadata>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_permissioned_server_metadata(
    ctx: Context<CreatePermissionedServerMetadataCtx>,
    server_url: String,
) -> Result<()> {
    let presale = ctx.accounts.presale.load()?;

    let whitelist_mode = WhitelistMode::from(presale.whitelist_mode);
    require!(
        whitelist_mode == WhitelistMode::PermissionWithMerkleProof
            || whitelist_mode == WhitelistMode::PermissionWithAuthority,
        PresaleError::InvalidPresaleWhitelistMode
    );

    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let progress = presale.get_presale_progress(current_timestamp);

    require!(
        progress == PresaleProgress::Ongoing || progress == PresaleProgress::NotStarted,
        PresaleError::PresaleEnded
    );

    ctx.accounts
        .permissioned_server_metadata
        .initialize(ctx.accounts.presale.key(), server_url.clone())?;

    emit_cpi!(EvtPermissionedServerMetadataCreate {
        presale: ctx.accounts.presale.key(),
        permissioned_server_metadata: ctx.accounts.permissioned_server_metadata.key(),
        server_url
    });

    Ok(())
}
