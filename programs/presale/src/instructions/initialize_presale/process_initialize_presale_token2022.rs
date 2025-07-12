use crate::{
    instructions::initialize_presale::{
        process_create_presale_vault::{
            process_create_presale_vault, ProcessCreatePresaleVaultArgs,
        },
        process_mint::{process_mint_token_supply, ProcessMintTokenSupplyArgs},
    },
    *,
};
use anchor_lang::system_program::Transfer;
use anchor_spl::token_interface::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: InitializePresaleArgs)]
pub struct InitializePresaleToken2022Ctx<'info> {
    #[account(
        init,
        signer,
        payer = payer,
        mint::decimals = params.token_info.decimals,
        mint::authority = presale_authority,
        mint::token_program = token_program,
        extensions::metadata_pointer::authority = presale_authority,
        extensions::metadata_pointer::metadata_address = mint,
    )]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

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

    /// CHECK: presale authority
    #[account(
       address = presale_authority::ID
    )]
    pub presale_authority: UncheckedAccount<'info>,

    pub quote_token_mint: Box<InterfaceAccount<'info, Mint>>,

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
    pub presale_vault: Box<InterfaceAccount<'info, TokenAccount>>,

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
    pub quote_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: creator
    pub creator: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_presale_token2022<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializePresaleToken2022Ctx<'info>>,
    args: &InitializePresaleArgs,
) -> Result<()> {
    // 1. Validate params
    args.validate()?;

    // 2. Ensure quote is whitelisted
    // TODO: Reevaluate due to need to support token2022
    // ensure_whitelisted_quote(ctx.accounts.quote_token_mint.key())?;

    // 3. Ensure quote token extensions are permissionless
    ensure_supported_token2022_extensions(&ctx.accounts.quote_token_mint)?;

    let InitializePresaleArgs {
        token_info,
        tokenomic,
        presale_params,
        locked_vesting_params,
    } = args;

    // 4. Create token metadata
    process_create_token_metadata(ProcessCreateTokenMetadataArgs {
        token_metadata_initialize_accounts: TokenMetadataInitialize {
            program_id: ctx.accounts.token_program.to_account_info(),
            metadata: ctx.accounts.mint.to_account_info(),
            update_authority: ctx.accounts.presale_authority.to_account_info(),
            mint_authority: ctx.accounts.presale_authority.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        },
        payer: ctx.accounts.payer.to_account_info(),
        token_info: token_info.clone(),
    })?;

    // 5. Mint token
    process_mint_token_supply(ProcessMintTokenSupplyArgs {
        mint: ctx.accounts.mint.to_account_info(),
        base_vault: ctx.accounts.presale_vault.to_account_info(),
        presale_authority: ctx.accounts.presale_authority.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        tokenomic,
    })?;

    // 6. Initialize vault
    let mint_pubkeys = InitializePresaleVaultAccountPubkeys {
        base_mint: ctx.accounts.mint.key(),
        quote_mint: ctx.accounts.quote_token_mint.key(),
        base_token_vault: ctx.accounts.presale_vault.key(),
        quote_token_vault: ctx.accounts.quote_token_vault.key(),
        owner: ctx.accounts.creator.key(),
    };
    process_create_presale_vault(ProcessCreatePresaleVaultArgs {
        presale: &ctx.accounts.presale,
        tokenomic_params: tokenomic,
        presale_params,
        locked_vesting_params: locked_vesting_params.as_ref(),
        mint_pubkeys,
        remaining_accounts: ctx.remaining_accounts,
    })?;

    emit_cpi!(EvtPresaleVaultCreate {
        base_mint: ctx.accounts.mint.key(),
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
        max_deposit_fee: presale_params.max_deposit_fee,
        deposit_fee_bps: presale_params.deposit_fee_bps,
    });

    Ok(())
}

fn transfer_rent_for_metadata_extension<'info>(
    mint_account: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
) -> Result<()> {
    let rent = Rent::get()?;
    let rent_exempt_balance = rent.minimum_balance(mint_account.data_len());
    let current_balance = mint_account.lamports();

    if current_balance < rent_exempt_balance {
        let required_lamport = rent_exempt_balance.checked_sub(current_balance).unwrap();
        anchor_lang::system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                Transfer {
                    from: payer,
                    to: mint_account,
                },
            ),
            required_lamport,
        )?;
    }

    Ok(())
}

struct ProcessCreateTokenMetadataArgs<'info> {
    token_metadata_initialize_accounts: TokenMetadataInitialize<'info>,
    token_info: TokenInfoArgs,
    payer: AccountInfo<'info>,
}

fn process_create_token_metadata(args: ProcessCreateTokenMetadataArgs) -> Result<()> {
    let ProcessCreateTokenMetadataArgs {
        token_metadata_initialize_accounts,
        token_info,
        payer,
    } = args;

    let TokenInfoArgs {
        name, symbol, uri, ..
    } = token_info;

    let metadata_account = token_metadata_initialize_accounts
        .metadata
        .to_account_info();

    let mint_account = token_metadata_initialize_accounts.mint.to_account_info();

    let token_program = token_metadata_initialize_accounts
        .program_id
        .to_account_info();

    let signer_seeds = &[&presale_authority_seeds!()[..]];
    token_metadata_initialize(
        CpiContext::new_with_signer(
            token_program,
            token_metadata_initialize_accounts,
            signer_seeds,
        ),
        name,
        symbol,
        uri,
    )?;

    // We need to transfer rent for the metadata extension
    // Token program initially initialize account with BASE size only. It will realloc and expand when metadata extension is added.
    transfer_rent_for_metadata_extension(mint_account, payer, metadata_account)?;

    Ok(())
}
