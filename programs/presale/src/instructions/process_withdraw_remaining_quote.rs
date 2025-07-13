use crate::*;
use anchor_spl::{
    token_2022::{transfer_checked, TransferChecked},
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
        has_one = owner
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    #[account(mut)]
    pub owner_quote_token: InterfaceAccount<'info, TokenAccount>,

    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_withdraw_remaining_quote(ctx: Context<WithdrawRemainingQuoteCtx>) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let mut escrow = ctx.accounts.escrow.load_mut()?;

    // 1. Ensure escrow haven't withdrawn remaining quote yet
    require!(
        !escrow.is_remaining_quote_withdrawn(),
        PresaleError::RemainingQuoteAlreadyWithdrawn
    );

    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let presale_progress = presale.get_presale_progress(current_timestamp);

    let presale_mode = PresaleMode::from(presale.presale_mode);

    // 2. Ensure presale is in failed or prorata completed state
    require!(
        presale_progress == PresaleProgress::Failed
            || (presale_progress == PresaleProgress::Completed
                && presale_mode == PresaleMode::Prorata),
        PresaleError::PresaleNotOpenForWithdrawRemainingQuote
    );

    let amount_to_refund = if presale_progress == PresaleProgress::Failed {
        // 3. Failed presale will refund all tokens to the owner
        escrow.get_total_deposit_amount_with_fees()?
    } else {
        // 4. Prorata will refund only the overflow (unused) quote token
        let remaining_quote_amount = presale
            .total_deposit
            .saturating_sub(presale.presale_maximum_cap);

        u128::from(escrow.total_deposit)
            .checked_mul(remaining_quote_amount.into())
            .unwrap()
            .checked_div(presale.total_deposit.into())
            .unwrap()
            .try_into()
            .unwrap()
    };

    require!(amount_to_refund > 0, PresaleError::ZeroTokenAmount);

    presale.update_total_refunded_quote_token(amount_to_refund)?;
    escrow.update_remaining_quote_withdrawn()?;

    let signer_seeds = &[&presale_authority_seeds!()[..]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.quote_token_vault.to_account_info(),
                to: ctx.accounts.owner_quote_token.to_account_info(),
                mint: ctx.accounts.quote_mint.to_account_info(),
                authority: ctx.accounts.presale_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount_to_refund,
        ctx.accounts.quote_mint.decimals,
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
