use crate::{
    instructions::create_escrow::process_create_escrow::{
        process_create_escrow, HandleCreateEscrowArgs,
    },
    *,
};

#[event_cpi]
#[derive(Accounts)]
pub struct CreatePermissionlessEscrowCtx<'info> {
    #[account(mut)]
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::ESCROW_PREFIX,
            presale.key().as_ref(),
            owner.key().as_ref(),
            &crate::constants::DEFAULT_PERMISSIONLESS_REGISTRY_INDEX.to_be_bytes(),
        ],
        bump,
        payer = payer,
        space = 8 + Escrow::INIT_SPACE
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    /// CHECK: Owner of the escrow account
    pub owner: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_permissionless_escrow(
    ctx: Context<CreatePermissionlessEscrowCtx>,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;

    // Ensure presale is permissionless
    let whitelist_mode = WhitelistMode::from(presale.whitelist_mode);
    require!(
        !whitelist_mode.is_permissioned(),
        PresaleError::InvalidPresaleWhitelistMode
    );

    process_create_escrow(HandleCreateEscrowArgs {
        presale: &mut presale,
        escrow: &ctx.accounts.escrow,
        presale_pubkey: ctx.accounts.presale.key(),
        owner_pubkey: ctx.accounts.owner.key(),
        registry_index: crate::constants::DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
    })?;

    emit_cpi!(EvtEscrowCreate {
        presale: ctx.accounts.presale.key(),
        owner: ctx.accounts.owner.key(),
        whitelist_mode: presale.whitelist_mode,
        total_escrow_count: presale.total_escrow,
    });

    Ok(())
}
