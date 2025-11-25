use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Default)]
pub struct CreateMerkleRootConfigParams {
    /// The 256-bit merkle root.
    pub root: [u8; 32],
    /// Version
    pub version: u64,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: CreateMerkleRootConfigParams)]
pub struct CreateMerkleRootConfigCtx<'info> {
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::MERKLE_ROOT_CONFIG_PREFIX,
            presale.key().as_ref(),
            params.version.to_le_bytes().as_ref()
        ],
        bump,
        payer = creator,
        space = 8 + MerkleRootConfig::INIT_SPACE
    )]
    pub merkle_root_config: AccountLoader<'info, MerkleRootConfig>,

    #[account(
        mut,
        constraint = presale.load()?.owner == creator.key() @ PresaleError::InvalidCreatorAccount
    )]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_merkle_root_config(
    ctx: Context<CreateMerkleRootConfigCtx>,
    params: CreateMerkleRootConfigParams,
) -> Result<()> {
    let presale = ctx.accounts.presale.load()?;

    let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;
    let presale_progress = presale.get_presale_progress(current_timestamp);

    // 1. Ensure presale is still in deposit phase
    require!(
        presale_progress == PresaleProgress::NotStarted
            || presale_progress == PresaleProgress::Ongoing,
        PresaleError::PresaleEnded
    );

    // 2. Ensure presale is permissioned with merkle proof
    let whitelist_mode: WhitelistMode = presale.whitelist_mode.safe_cast()?;
    require!(
        whitelist_mode == WhitelistMode::PermissionWithMerkleProof,
        PresaleError::InvalidPresaleWhitelistMode
    );

    let mut merkle_root_config = ctx.accounts.merkle_root_config.load_init()?;
    merkle_root_config.initialize(ctx.accounts.presale.key(), params.root, params.version);

    emit_cpi!(EvtMerkleRootConfigCreate {
        presale: ctx.accounts.presale.key(),
        owner: ctx.accounts.creator.key(),
        config: ctx.accounts.merkle_root_config.key(),
        version: params.version,
        root: params.root,
    });

    Ok(())
}
