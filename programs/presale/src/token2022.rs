use crate::*;
use anchor_spl::{
    memo::{self, BuildMemo, Memo},
    token::Token,
    token_2022::spl_token_2022::{
        self,
        extension::{
            self,
            transfer_fee::{TransferFee, MAX_FEE_BASIS_POINTS},
            BaseStateWithExtensions, ExtensionType, StateWithExtensions,
        },
    },
    token_interface::{Mint, TokenAccount, TokenInterface},
};

pub const PRESALE_MEMO: &[u8] = b"Presale";

pub fn is_supported_mint(mint_account: &InterfaceAccount<Mint>) -> Result<bool> {
    let mint_info = mint_account.to_account_info();
    if *mint_info.owner == Token::id() {
        return Ok(true);
    }

    if spl_token_2022::native_mint::check_id(&mint_account.key()) {
        return Ok(false);
    }

    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;
    let extensions = mint.get_extension_types()?;
    for e in extensions {
        if e != ExtensionType::TransferFeeConfig
            && e != ExtensionType::MetadataPointer
            && e != ExtensionType::TokenMetadata
            && e != ExtensionType::TransferHook
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn ensure_supported_token2022_extensions(mint_account: &InterfaceAccount<Mint>) -> Result<()> {
    require!(
        is_supported_mint(mint_account)?,
        PresaleError::UnsupportedToken2022MintOrExtension
    );
    Ok(())
}

/// refer code from Orca
#[derive(Debug)]
pub struct TransferFeeIncludedAmount {
    pub amount: u64,
    pub transfer_fee: u64,
}

#[derive(Debug)]
pub struct TransferFeeExcludedAmount {
    pub amount: u64,
    pub transfer_fee: u64,
}

pub fn calculate_transfer_fee_excluded_amount<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
    transfer_fee_included_amount: u64,
) -> Result<TransferFeeExcludedAmount> {
    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        let transfer_fee = epoch_transfer_fee
            .calculate_fee(transfer_fee_included_amount)
            .ok_or_else(|| PresaleError::MathOverflow)?;
        let transfer_fee_excluded_amount = transfer_fee_included_amount
            .checked_sub(transfer_fee)
            .ok_or_else(|| PresaleError::MathOverflow)?;
        return Ok(TransferFeeExcludedAmount {
            amount: transfer_fee_excluded_amount,
            transfer_fee,
        });
    }

    Ok(TransferFeeExcludedAmount {
        amount: transfer_fee_included_amount,
        transfer_fee: 0,
    })
}

pub fn calculate_transfer_fee_included_amount<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
    transfer_fee_excluded_amount: u64,
) -> Result<TransferFeeIncludedAmount> {
    if transfer_fee_excluded_amount == 0 {
        return Ok(TransferFeeIncludedAmount {
            amount: 0,
            transfer_fee: 0,
        });
    }

    if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
        let transfer_fee: u64 =
            if u16::from(epoch_transfer_fee.transfer_fee_basis_points) == MAX_FEE_BASIS_POINTS {
                // edge-case: if transfer fee rate is 100%, current SPL implementation returns 0 as inverse fee.
                // https://github.com/solana-labs/solana-program-library/blob/fe1ac9a2c4e5d85962b78c3fc6aaf028461e9026/token/program-2022/src/extension/transfer_fee/mod.rs#L95

                // But even if transfer fee is 100%, we can use maximum_fee as transfer fee.
                // if transfer_fee_excluded_amount + maximum_fee > u64 max, the following checked_add should fail.
                u64::from(epoch_transfer_fee.maximum_fee)
            } else {
                epoch_transfer_fee
                    .calculate_inverse_fee(transfer_fee_excluded_amount)
                    .ok_or(PresaleError::MathOverflow)?
            };

        let transfer_fee_included_amount = transfer_fee_excluded_amount
            .checked_add(transfer_fee)
            .ok_or(PresaleError::MathOverflow)?;

        // verify transfer fee calculation for safety
        let transfer_fee_verification = epoch_transfer_fee
            .calculate_fee(transfer_fee_included_amount)
            .unwrap();
        if transfer_fee != transfer_fee_verification {
            // We believe this should never happen
            unreachable!("Fee inverse is incorrect");
        }

        return Ok(TransferFeeIncludedAmount {
            amount: transfer_fee_included_amount,
            transfer_fee,
        });
    }

    Ok(TransferFeeIncludedAmount {
        amount: transfer_fee_excluded_amount,
        transfer_fee: 0,
    })
}

pub fn get_epoch_transfer_fee<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
) -> Result<Option<TransferFee>> {
    let token_mint_info = token_mint.to_account_info();
    if *token_mint_info.owner == Token::id() {
        return Ok(None);
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked =
        StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    if let Ok(transfer_fee_config) =
        token_mint_unpacked.get_extension::<extension::transfer_fee::TransferFeeConfig>()
    {
        let epoch = Clock::get()?.epoch;
        return Ok(Some(transfer_fee_config.get_epoch_fee(epoch).clone()));
    }

    Ok(None)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum AccountsType {
    TransferHookBase,
    TransferHookQuote,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RemainingAccountsInfo {
    pub slices: Vec<RemainingAccountsSlice>,
}

#[derive(Debug, Default)]
pub struct ParsedRemainingTransferHookAccounts<'a, 'info> {
    pub transfer_hook_base: Option<&'a [AccountInfo<'info>]>,
    pub transfer_hook_quote: Option<&'a [AccountInfo<'info>]>,
}

/// Parse remaining accounts by consume all the transfer hooks related accounts.
pub fn parse_remaining_accounts_for_transfer_hook<'c: 'info, 'info>(
    remaining_accounts: &mut &'c [AccountInfo<'info>],
    remaining_accounts_slice: &[RemainingAccountsSlice],
    valid_accounts_type_list: &[AccountsType],
) -> Result<ParsedRemainingTransferHookAccounts<'c, 'info>> {
    let mut parsed_remaining_accounts = ParsedRemainingTransferHookAccounts::default();

    if remaining_accounts_slice.is_empty() {
        return Ok(ParsedRemainingTransferHookAccounts::default());
    }

    for slice in remaining_accounts_slice.iter() {
        if !valid_accounts_type_list.contains(&slice.accounts_type) {
            return Err(PresaleError::InvalidRemainingAccountSlice.into());
        }

        if slice.length == 0 {
            continue;
        }

        if remaining_accounts.len() < slice.length as usize {
            return Err(PresaleError::InvalidRemainingAccountSlice.into());
        }

        let end_idx = slice.length as usize;
        let accounts = &remaining_accounts[0..end_idx];
        *remaining_accounts = &remaining_accounts[end_idx..];

        match slice.accounts_type {
            AccountsType::TransferHookBase => {
                if parsed_remaining_accounts.transfer_hook_base.is_some() {
                    return Err(PresaleError::DuplicatedRemainingAccountTypes.into());
                }
                parsed_remaining_accounts.transfer_hook_base = Some(accounts);
            }
            AccountsType::TransferHookQuote => {
                if parsed_remaining_accounts.transfer_hook_quote.is_some() {
                    return Err(PresaleError::DuplicatedRemainingAccountTypes.into());
                }
                parsed_remaining_accounts.transfer_hook_quote = Some(accounts);
            }
        }
    }

    Ok(parsed_remaining_accounts)
}

fn is_transfer_memo_required(user_token_ai: &AccountInfo<'_>) -> Result<bool> {
    if user_token_ai.owner.eq(&anchor_spl::token::ID) {
        return Ok(false);
    }

    let account_data = user_token_ai.try_borrow_data()?;
    let token_account_unpacked =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account_data)?;

    let memo_transfer_ext =
        token_account_unpacked.get_extension::<extension::memo_transfer::MemoTransfer>();

    if let Ok(memo_transfer) = memo_transfer_ext {
        Ok(memo_transfer.require_incoming_transfer_memos.into())
    } else {
        Ok(false)
    }
}

fn get_transfer_hook_program_id<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
) -> Result<Option<Pubkey>> {
    let token_mint_info = token_mint.to_account_info();
    if *token_mint_info.owner == Token::id() {
        return Ok(None);
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked =
        StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    Ok(extension::transfer_hook::get_program_id(
        &token_mint_unpacked,
    ))
}

#[derive(Clone, Copy)]
pub struct MemoTransferContext<'a, 'info> {
    pub memo_program: &'a Program<'info, Memo>,
    pub memo: &'static [u8],
}

pub fn transfer_from_user<'a, 'c: 'info, 'info>(
    authority: &'a Signer<'info>,
    token_mint: &'a InterfaceAccount<'info, Mint>,
    token_owner_account: &'a InterfaceAccount<'info, TokenAccount>,
    destination_token_account: &'a InterfaceAccount<'info, TokenAccount>,
    token_program: &'a Interface<'info, TokenInterface>,
    amount: u64,
    memo_transfer_context: Option<MemoTransferContext<'a, 'info>>,
    transfer_hook_accounts: Option<&'c [AccountInfo<'info>]>,
) -> Result<()> {
    let destination_account = destination_token_account.to_account_info();

    if let Some(memo_ctx) = memo_transfer_context {
        if is_transfer_memo_required(&destination_account)? {
            let ctx = CpiContext::new(memo_ctx.memo_program.to_account_info(), BuildMemo {});
            memo::build_memo(ctx, memo_ctx.memo)?;
        }
    }

    let mut instruction = spl_token_2022::instruction::transfer_checked(
        token_program.key,
        &token_owner_account.key(),
        &token_mint.key(),
        destination_account.key,
        authority.key,
        &[],
        amount,
        token_mint.decimals,
    )?;

    let mut account_infos = vec![
        token_owner_account.to_account_info(),
        token_mint.to_account_info(),
        destination_account.to_account_info(),
        authority.to_account_info(),
    ];

    if let Some(hook_program_id) = get_transfer_hook_program_id(token_mint)? {
        let Some(transfer_hook_accounts) = transfer_hook_accounts else {
            return Err(PresaleError::MissingRemainingAccountForTransferHook.into());
        };

        spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi(
            &mut instruction,
            &mut account_infos,
            &hook_program_id,
            token_owner_account.to_account_info(),
            token_mint.to_account_info(),
            destination_account.to_account_info(),
            authority.to_account_info(),
            amount,
            transfer_hook_accounts,
        )?;
    } else {
        require!(
            transfer_hook_accounts.is_none(),
            PresaleError::NoTransferHookProgram
        );
    }

    anchor_lang::solana_program::program::invoke_signed(&instruction, &account_infos, &[])?;

    Ok(())
}

pub fn transfer_from_presale_to_user<'c: 'info, 'info>(
    presale_authority: &UncheckedAccount<'info>,
    token_mint: &InterfaceAccount<'info, Mint>,
    token_vault: &InterfaceAccount<'info, TokenAccount>,
    token_owner_account: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Interface<'info, TokenInterface>,
    amount: u64,
    memo_transfer_context: Option<MemoTransferContext<'_, 'info>>,
    transfer_hook_accounts: Option<&'c [AccountInfo<'info>]>,
) -> Result<()> {
    let signer_seeds = &[&presale_authority_seeds!()[..]];

    let destination_account = token_owner_account.to_account_info();

    if let Some(memo_ctx) = memo_transfer_context {
        if is_transfer_memo_required(&destination_account)? {
            let ctx = CpiContext::new(memo_ctx.memo_program.to_account_info(), BuildMemo {});
            memo::build_memo(ctx, memo_ctx.memo)?;
        }
    }

    let mut instruction = spl_token_2022::instruction::transfer_checked(
        token_program.key,
        &token_vault.key(),
        &token_mint.key(),
        &token_owner_account.key(),
        &presale_authority.key(),
        &[],
        amount,
        token_mint.decimals,
    )?;

    let mut account_infos = vec![
        token_vault.to_account_info(),
        token_mint.to_account_info(),
        token_owner_account.to_account_info(),
        presale_authority.to_account_info(),
    ];

    if let Some(hook_program_id) = get_transfer_hook_program_id(token_mint)? {
        let Some(transfer_hook_accounts) = transfer_hook_accounts else {
            return Err(PresaleError::MissingRemainingAccountForTransferHook.into());
        };

        spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi(
            &mut instruction,
            &mut account_infos,
            &hook_program_id,
            token_vault.to_account_info(),
            token_mint.to_account_info(),
            token_owner_account.to_account_info(),
            presale_authority.to_account_info(),
            amount,
            transfer_hook_accounts,
        )?;
    } else {
        require!(
            transfer_hook_accounts.is_none(),
            PresaleError::NoTransferHookProgram
        );
    }

    anchor_lang::solana_program::program::invoke_signed(
        &instruction,
        &account_infos,
        signer_seeds,
    )?;

    Ok(())
}
