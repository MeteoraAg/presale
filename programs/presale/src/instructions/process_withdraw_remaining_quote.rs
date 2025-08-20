use crate::*;
use anchor_spl::{
    memo::Memo,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawRemainingQuoteCtx<'info> {
    #[account(
        mut,
        has_one = quote_token_vault,
        has_one = quote_mint,
    )]
    pub presale: AccountLoader<'info, Presale>,

    #[account(mut)]
    pub quote_token_vault: InterfaceAccount<'info, TokenAccount>,
    pub quote_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: The presale authority is the PDA of the presale.
    #[account(
        address = crate::const_pda::presale_authority::ID,
    )]
    pub presale_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        has_one = presale,
        has_one = owner
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    #[account(mut)]
    pub owner_quote_token: InterfaceAccount<'info, TokenAccount>,

    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,

    pub memo_program: Program<'info, Memo>,
}

pub fn handle_withdraw_remaining_quote<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, WithdrawRemainingQuoteCtx<'info>>,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let mut escrow = ctx.accounts.escrow.load_mut()?;

    // 1. Ensure escrow haven't withdrawn remaining quote yet
    require!(
        !escrow.is_remaining_quote_withdrawn(),
        PresaleError::RemainingQuoteAlreadyWithdrawn
    );

    let current_timestamp = Clock::get()?.unix_timestamp as u64;

    // 2. Ensure the presale is in a state that allows withdrawing remaining quote
    let amount_to_refund =
        presale.validate_and_get_escrow_remaining_quote(&escrow, current_timestamp)?;

    require!(amount_to_refund > 0, PresaleError::ZeroTokenAmount);

    // 3. Update presale and escrow state
    presale.update_total_refunded_quote_token(amount_to_refund, escrow.registry_index)?;
    escrow.update_remaining_quote_withdrawn()?;

    let transfer_hook_accounts = parse_remaining_accounts_for_transfer_hook(
        &mut &ctx.remaining_accounts[..],
        &remaining_accounts_info.slices,
        &[AccountsType::TransferHookQuote],
    )?;

    transfer_from_presale_to_user(
        &ctx.accounts.presale_authority,
        &ctx.accounts.quote_mint,
        &ctx.accounts.quote_token_vault,
        &ctx.accounts.owner_quote_token,
        &ctx.accounts.token_program,
        amount_to_refund,
        Some(MemoTransferContext {
            memo_program: &ctx.accounts.memo_program,
            memo: PRESALE_MEMO,
        }),
        transfer_hook_accounts.transfer_hook_quote,
    )?;

    let exclude_fee_amount_to_refund =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.quote_mint, amount_to_refund)?.amount;

    emit_cpi!(EvtWithdrawRemainingQuote {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        owner: ctx.accounts.owner.key(),
        amount_refunded: exclude_fee_amount_to_refund,
        presale_total_refunded_quote_token: presale.total_refunded_quote_token,
    });

    Ok(())
}
