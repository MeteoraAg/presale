use crate::{
    instructions::initialize_presale::process_create_presale_vault::{
        process_create_presale_vault, ProcessCreatePresaleVaultArgs,
    },
    *,
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: InitializePresaleArgs)]
pub struct InitializePresaleCtx<'info> {
    pub presale_mint: InterfaceAccount<'info, Mint>,

    /// presale address
    #[account(
        init,
        seeds = [
            crate::constants::seeds::PRESALE_PREFIX.as_ref(),
            base.key().as_ref(),
            presale_mint.key().as_ref(),
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

    pub quote_token_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::BASE_VAULT_PREFIX.as_ref(),
            presale.key().as_ref(),
        ],
        bump,
        payer = payer,
        token::mint = presale_mint,
        token::authority = presale_authority,
        token::token_program = base_token_program
    )]
    pub presale_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        seeds = [
            crate::constants::seeds::QUOTE_VAULT_PREFIX.as_ref(),
            presale.key().as_ref(),
        ],
        bump,
        payer = payer,
        token::mint = quote_token_mint,
        token::authority = presale_authority,
        token::token_program = quote_token_program
    )]
    pub quote_token_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub payer_presale_token: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: creator
    pub creator: UncheckedAccount<'info>,

    pub base: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_presale<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
    args: InitializePresaleArgs,
    remaining_account_info: RemainingAccountsInfo,
) -> Result<()> {
    // 1. Validate params
    args.validate()?;

    let mut remaining_account_slice = &ctx.remaining_accounts[..];

    // 2. Ensure base and quote token extensions are permissionless
    ensure_supported_token2022_extensions(&ctx.accounts.quote_token_mint)?;
    ensure_supported_token2022_extensions(&ctx.accounts.presale_mint)?;

    let InitializePresaleArgs {
        presale_params,
        locked_vesting_params,
        presale_registries,
        ..
    } = args;

    // 3. Initialize vault
    let mint_pubkeys = InitializePresaleVaultAccountPubkeys {
        base_mint: ctx.accounts.presale_mint.key(),
        quote_mint: ctx.accounts.quote_token_mint.key(),
        base_token_vault: ctx.accounts.presale_vault.key(),
        quote_token_vault: ctx.accounts.quote_token_vault.key(),
        owner: ctx.accounts.creator.key(),
        base: ctx.accounts.base.key(),
        base_token_program: ctx.accounts.base_token_program.key(),
        quote_token_program: ctx.accounts.quote_token_program.key(),
    };

    let locked_vesting_params = locked_vesting_params.option();

    process_create_presale_vault(ProcessCreatePresaleVaultArgs {
        presale: &ctx.accounts.presale,
        presale_params: &presale_params,
        presale_registries: &presale_registries,
        locked_vesting_params: locked_vesting_params.as_ref(),
        mint_pubkeys,
        remaining_accounts: &mut remaining_account_slice,
    })?;

    let presale_pool_supply = presale_registries
        .iter()
        .try_fold(0u64, |acc, reg| acc.safe_add(reg.presale_supply))?;

    let include_fee_presale_pool_supply =
        calculate_transfer_fee_included_amount(&ctx.accounts.presale_mint, presale_pool_supply)?
            .amount;

    let transfer_hook_accounts = parse_remaining_accounts_for_transfer_hook(
        &mut remaining_account_slice,
        &remaining_account_info.slices,
        &[AccountsType::TransferHookBase],
    )?;

    // 4. Transfer token to presale vault
    transfer_from_user(
        &ctx.accounts.payer,
        &ctx.accounts.presale_mint,
        &ctx.accounts.payer_presale_token,
        &ctx.accounts.presale_vault,
        &ctx.accounts.base_token_program,
        include_fee_presale_pool_supply,
        None,
        transfer_hook_accounts.transfer_hook_base,
    )?;

    emit_cpi!(EvtPresaleVaultCreate {
        base_mint: ctx.accounts.presale_mint.key(),
        quote_mint: ctx.accounts.quote_token_mint.key(),
        presale_registries,
        lock_duration: locked_vesting_params
            .as_ref()
            .map(|p| p.lock_duration)
            .unwrap_or(0),
        vest_duration: locked_vesting_params
            .as_ref()
            .map(|p| p.vest_duration)
            .unwrap_or(0),
        whitelist_mode: presale_params.whitelist_mode,
        presale_mode: presale_params.presale_mode,
        presale_start_time: presale_params.presale_start_time,
        presale_end_time: presale_params.presale_end_time,
        presale_maximum_cap: presale_params.presale_maximum_cap,
        presale_minimum_cap: presale_params.presale_minimum_cap,
    });

    Ok(())
}
