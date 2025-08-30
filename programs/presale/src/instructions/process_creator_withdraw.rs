use std::collections::BTreeSet;

use anchor_spl::{
    memo::Memo,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CreatorWithdrawCtx<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub presale: AccountLoader<'info, Presale>,

    /// CHECK: This is the presale authority
    #[account(
        address = crate::presale_authority::ID,
    )]
    pub presale_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub owner_token: InterfaceAccount<'info, TokenAccount>,
    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,

    pub memo_program: Program<'info, Memo>,
}

#[derive(Accounts)]
pub struct CreatorWithdrawQuoteCtx<'info> {
    #[account(mut)]
    pub quote_token_vault: InterfaceAccount<'info, TokenAccount>,
    pub quote_mint: InterfaceAccount<'info, Mint>,
}

impl<'info> CreatorWithdrawQuoteCtx<'info> {
    pub fn try_accounts_and_validate<'c: 'info>(
        presale: &Presale,
        remaining_accounts: &mut &'c [AccountInfo<'info>],
    ) -> Result<Self> {
        let accounts = Self::try_accounts(
            &crate::ID,
            remaining_accounts,
            &[],
            &mut CreatorWithdrawQuoteCtxBumps {},
            &mut BTreeSet::new(),
        )?;

        require!(
            accounts.quote_token_vault.key() == presale.quote_token_vault,
            PresaleError::InvalidTokenVault
        );

        require!(
            accounts.quote_mint.key() == presale.quote_mint,
            PresaleError::InvalidQuoteMint
        );

        Ok(accounts)
    }
}

#[derive(Accounts)]
pub struct CreatorWithdrawBaseCtx<'info> {
    #[account(mut)]
    pub base_token_vault: InterfaceAccount<'info, TokenAccount>,
    pub base_mint: InterfaceAccount<'info, Mint>,
}

impl<'info> CreatorWithdrawBaseCtx<'info> {
    pub fn try_accounts_and_validate<'c: 'info>(
        presale: &Presale,
        remaining_accounts: &mut &'c [AccountInfo<'info>],
    ) -> Result<Self> {
        let accounts = Self::try_accounts(
            &crate::ID,
            remaining_accounts,
            &[],
            &mut CreatorWithdrawBaseCtxBumps {},
            &mut BTreeSet::new(),
        )?;

        require!(
            accounts.base_token_vault.key() == presale.base_token_vault,
            PresaleError::InvalidTokenVault
        );

        require!(
            accounts.base_mint.key() == presale.base_mint,
            PresaleError::InvalidBaseMint
        );

        Ok(accounts)
    }
}

pub fn handle_creator_withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, CreatorWithdrawCtx<'info>>,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;

    let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;
    let presale_progress = presale.get_presale_progress(current_timestamp);

    // 1. Ensure presale is completed
    require!(
        presale_progress == PresaleProgress::Completed
            || presale_progress == PresaleProgress::Failed,
        PresaleError::PresaleNotOpenForWithdraw
    );

    // 2. Ensure creator haven't withdrawn yet
    require!(
        !presale.has_creator_withdrawn(),
        PresaleError::CreatorAlreadyWithdrawn
    );

    presale.update_creator_withdrawn()?;

    let mut remaining_account_slice = &ctx.remaining_accounts[..];

    let (amount, from, mint, valid_accounts_type_list) = match presale_progress {
        PresaleProgress::Completed => {
            // 3. Presale is completed, withdraw quote token
            let CreatorWithdrawQuoteCtx {
                quote_mint,
                quote_token_vault,
            } = CreatorWithdrawQuoteCtx::try_accounts_and_validate(
                &presale,
                &mut remaining_account_slice,
            )?;

            (
                // Prorata can have total_deposit > presale_maximum_cap
                presale.total_deposit.min(presale.presale_maximum_cap),
                quote_token_vault,
                quote_mint,
                [AccountsType::TransferHookQuote],
            )
        }
        PresaleProgress::Failed => {
            // 4. Presale failed, withdraw base token
            let CreatorWithdrawBaseCtx {
                base_mint,
                base_token_vault,
            } = CreatorWithdrawBaseCtx::try_accounts_and_validate(
                &presale,
                &mut remaining_account_slice,
            )?;

            (
                presale.presale_supply,
                base_token_vault,
                base_mint,
                [AccountsType::TransferHookBase],
            )
        }
        _ => {
            return Err(PresaleError::UndeterminedError.into());
        }
    };

    let transfer_hook_accounts = parse_remaining_accounts_for_transfer_hook(
        &mut remaining_account_slice,
        &remaining_accounts_info.slices,
        &valid_accounts_type_list,
    )?;

    let transfer_hook_accounts = if presale_progress == PresaleProgress::Completed {
        transfer_hook_accounts.transfer_hook_quote
    } else {
        transfer_hook_accounts.transfer_hook_base
    };

    transfer_from_presale_to_user(
        &ctx.accounts.presale_authority,
        &mint,
        &from,
        &ctx.accounts.owner_token,
        &ctx.accounts.token_program,
        amount,
        Some(MemoTransferContext {
            memo_program: &ctx.accounts.memo_program,
            memo: PRESALE_MEMO,
        }),
        transfer_hook_accounts,
    )?;

    let exclude_fee_amount = calculate_transfer_fee_excluded_amount(&mint, amount)?.amount;

    emit_cpi!(EvtCreatorWithdraw {
        presale: ctx.accounts.presale.key(),
        amount: exclude_fee_amount,
        presale_progress: presale_progress.into(),
        creator: ctx.accounts.owner.key(),
    });

    Ok(())
}
