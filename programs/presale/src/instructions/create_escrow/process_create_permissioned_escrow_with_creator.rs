use crate::{
    instructions::create_escrow::process_create_escrow::{
        process_create_escrow, HandleCreateEscrowArgs,
    },
    *,
};

#[event_cpi]
#[derive(Accounts)]
pub struct CreatePermissionedEscrowWithCreatorCtx<'info> {
    #[account(mut)]
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::ESCROW_PREFIX,
            presale.key().as_ref(),
            owner.key().as_ref()
        ],
        bump,
        payer = payer,
        space = 8 + Escrow::INIT_SPACE
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    /// CHECK: Owner of the escrow account
    pub owner: UncheckedAccount<'info>,

    pub operator: AccountLoader<'info, Operator>,

    pub operator_owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_permissioned_escrow_with_creator(
    ctx: Context<CreatePermissionedEscrowWithCreatorCtx>,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;

    // 1. Ensure presale is permissioned with authority
    let whitelist_mode = WhitelistMode::from(presale.whitelist_mode);
    require!(
        whitelist_mode == WhitelistMode::PermissionWithAuthority,
        PresaleError::InvalidPresaleWhitelistMode
    );

    // 2. Ensure rightful operator is provided
    let operator = ctx.accounts.operator.load()?;
    ensure_operator_belongs_to_presale_creator(
        &presale,
        &operator,
        &ctx.accounts.operator_owner.key(),
    )?;

    process_create_escrow(HandleCreateEscrowArgs {
        presale: &mut presale,
        escrow: &ctx.accounts.escrow,
        presale_pubkey: ctx.accounts.presale.key(),
        owner_pubkey: ctx.accounts.owner.key(),
    })?;

    emit_cpi!(EvtEscrowCreate {
        presale: ctx.accounts.presale.key(),
        owner: ctx.accounts.owner.key(),
        whitelist_mode: presale.whitelist_mode,
        total_escrow_count: presale.total_escrow,
    });

    Ok(())
}

fn ensure_operator_belongs_to_presale_creator(
    presale: &Presale,
    operator: &Operator,
    operator_owner: &Pubkey,
) -> Result<()> {
    require!(
        operator.owner == *operator_owner,
        PresaleError::InvalidOperator
    );

    require!(
        operator.creator == presale.owner,
        PresaleError::InvalidOperator
    );
    Ok(())
}
