use crate::{
    instructions::initialize_presale::process_create_presale_vault::{
        process_create_presale_vault, ProcessCreatePresaleVaultArgs,
    },
    *,
};
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::{BaseStateWithExtensions, PodStateWithExtensions},
        pod::PodMint,
    },
    token_interface::{spl_token_metadata_interface::state::TokenMetadata, *},
};
use mpl_token_metadata::accounts::Metadata;

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
    args: &InitializePresaleArgs,
    remaining_account_info: RemainingAccountsInfo,
) -> Result<()> {
    // 1. Validate params
    args.validate()?;

    let mut remaining_account_slice = &ctx.remaining_accounts[..];

    // 2. Ensure metadata is created
    match ctx.accounts.base_token_program.key() {
        anchor_spl::token::ID => {
            ensure_mint_metadata_initialized(
                &mut remaining_account_slice,
                &ctx.accounts.presale_mint.key(),
            )?;
        }
        anchor_spl::token_2022::ID => {
            ensure_mint_metadata_initialized_token_2022(
                &ctx.accounts.presale_mint.to_account_info(),
            )?;
        }
        _ => {
            unreachable!(
                "Unsupported token program: {}",
                ctx.accounts.base_token_program.key()
            );
        }
    }

    // 3. Ensure base and quote token extensions are permissionless
    ensure_supported_token2022_extensions(&ctx.accounts.quote_token_mint)?;
    ensure_supported_token2022_extensions(&ctx.accounts.presale_mint)?;

    let InitializePresaleArgs {
        tokenomic,
        presale_params,
        locked_vesting_params,
    } = args;

    // 4. Initialize vault
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

    process_create_presale_vault(ProcessCreatePresaleVaultArgs {
        presale: &ctx.accounts.presale,
        tokenomic_params: tokenomic,
        presale_params,
        locked_vesting_params: locked_vesting_params.as_ref(),
        mint_pubkeys,
        remaining_accounts: &mut remaining_account_slice,
    })?;

    let include_fee_presale_pool_supply = calculate_transfer_fee_included_amount(
        &ctx.accounts.presale_mint,
        tokenomic.presale_pool_supply,
    )?
    .amount;

    let transfer_hook_accounts = parse_remaining_accounts_for_transfer_hook(
        &mut remaining_account_slice,
        &remaining_account_info.slices,
        &[AccountsType::TransferHookBase],
    )?;

    // 5. Transfer token to presale vault
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
        buyer_maximum_deposit_cap: presale_params.buyer_maximum_deposit_cap,
        buyer_minimum_deposit_cap: presale_params.buyer_minimum_deposit_cap,
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

fn ensure_mint_metadata_initialized<'a, 'info>(
    remaining_accounts: &mut &[AccountInfo<'info>],
    mint_key: &Pubkey,
) -> Result<()> {
    let mpl_token_metadata_ai = remaining_accounts.split_first();

    let Some((mpl_token_metadata_ai, new_remaining_account_slice)) = mpl_token_metadata_ai else {
        return Err(PresaleError::InvalidMintMetadata.into());
    };

    *remaining_accounts = new_remaining_account_slice;

    let expected_metadata_pubkey = Metadata::find_pda(mint_key).0;
    require!(
        expected_metadata_pubkey == mpl_token_metadata_ai.key(),
        PresaleError::InvalidMintMetadata
    );

    // Make sure metadata is initialized by deserialize and check the content
    let metadata_state = Metadata::safe_deserialize(&mpl_token_metadata_ai.try_borrow_data()?)?;
    require!(
        metadata_state.mint == *mint_key,
        PresaleError::InvalidMintMetadata
    );

    Ok(())
}

fn ensure_mint_metadata_initialized_token_2022<'a, 'info>(
    mint: &'a AccountInfo<'info>,
) -> Result<()> {
    let buffer = mint.try_borrow_data()?;
    let mint = PodStateWithExtensions::<PodMint>::unpack(&buffer)?;
    mint.get_variable_len_extension::<TokenMetadata>()
        .map_err(|_| PresaleError::InvalidMintMetadata)?;
    Ok(())
}
