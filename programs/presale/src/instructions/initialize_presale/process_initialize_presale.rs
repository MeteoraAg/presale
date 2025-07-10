use crate::{
    instructions::initialize_presale::{
        process_create_metaplex_metadata::*,
        process_create_presale_vault::{
            process_create_presale_vault, ProcessCreatePresaleVaultArgs,
        },
        process_mint::{process_mint_token_supply, ProcessMintTokenSupplyArgs},
    },
    *,
};
use anchor_spl::token::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: InitializePresaleArgs)]
pub struct InitializePresaleCtx<'info> {
    #[account(
        init,
        payer = payer,
        mint::decimals = params.token_info.decimals,
        mint::authority = presale_authority,
    )]
    pub mint: Box<Account<'info, Mint>>,

    /// CHECK: mint_metadata
    #[account(mut)]
    pub mint_metadata: UncheckedAccount<'info>,

    /// CHECK: Metadata program
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,

    /// presale address
    #[account(
        init,
        seeds = [
            crate::constants::seeds::PRESALE_PREFIX.as_ref(),
            mint.key().as_ref(),
            quote_token_mint.key().as_ref(),
        ],
        bump,
        payer = payer,
        space = 8 + Presale::INIT_SPACE
    )]
    pub presale: AccountLoader<'info, Presale>,

    /// CHECK: presale_authority
    #[account(
       address = presale_authority::ID
    )]
    pub presale_authority: UncheckedAccount<'info>,

    pub quote_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::BASE_VAULT_PREFIX.as_ref(),
            presale.key().as_ref(),            
        ],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = presale_authority
    )]
    pub presale_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::QUOTE_VAULT_PREFIX.as_ref(),
            presale.key().as_ref(),            
        ],
        bump,
        payer = payer,
        token::mint = quote_token_mint,
        token::authority = presale_authority
    )]
    pub quote_token_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: creator
    pub creator: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_token_and_create_presale_vault<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
    args: &InitializePresaleArgs,
) -> Result<()> {
    // 1. Validate params
    args.validate()?;

    // 2. Ensure quote is whitelisted
    // ensure_whitelisted_quote(ctx.accounts.quote_token_mint.key())?;

    let InitializePresaleArgs {
        token_info,
        tokenomic,
        presale_params,
        locked_vesting_params,
    } = args;

    // 3. Create MPL metadata
    process_create_mpl_token_metadata(ProcessCreateTokenMetadataArgs {
        system_program: ctx.accounts.system_program.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        presale_authority: ctx.accounts.presale_authority.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        metadata_program: ctx.accounts.metadata_program.to_account_info(),
        mint_metadata: ctx.accounts.mint_metadata.to_account_info(),
        name: &token_info.name,
        symbol: &token_info.symbol,
        uri: &token_info.uri,
    })?;

    // 4. Mint token
    process_mint_token_supply(ProcessMintTokenSupplyArgs {
        mint: ctx.accounts.mint.to_account_info(),
        base_vault: ctx.accounts.presale_vault.to_account_info(),
        presale_authority: ctx.accounts.presale_authority.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        tokenomic,
    })?;

    // 5. Initialize vault
    process_create_presale_vault(ProcessCreatePresaleVaultArgs {
        presale: &ctx.accounts.presale,
        tokenomic_params: tokenomic,
        presale_params,
        locked_vesting_params: locked_vesting_params.as_ref(),
        remaining_accounts: ctx.remaining_accounts,
    })?;

    Ok(())
}
