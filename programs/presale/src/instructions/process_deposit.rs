use anchor_spl::{
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct DepositCtx<'info> {
    #[account(
        mut,
        has_one = quote_token_vault,
        has_one = quote_mint,
    )]
    pub presale: AccountLoader<'info, Presale>,
    #[account(mut)]
    pub quote_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        has_one = presale,
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    #[account(mut)]
    pub payer_quote_token: Box<InterfaceAccount<'info, TokenAccount>>,
    pub payer: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

// Max amount doesn't include the deposit and transfer fees. This is the maximum amount the user wants to deposit.
pub fn handle_deposit(ctx: Context<DepositCtx>, max_amount: u64) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let mut escrow = ctx.accounts.escrow.load_mut()?;

    // 1. Ensure presale is open for deposit
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let progress = presale.get_presale_progress(current_timestamp);
    require!(
        progress == PresaleProgress::Ongoing,
        PresaleError::PresaleNotOpenForDeposit
    );

    // 2. Ensure deposit amount is within the cap
    let presale_mode = PresaleMode::from(presale.presale_mode);
    let presale_handler = get_presale_mode_handler(presale_mode);
    let remaining_deposit_quota = presale_handler.get_remaining_deposit_quota(&presale, &escrow)?;
    let deposit_amount = remaining_deposit_quota.min(max_amount);

    require!(deposit_amount > 0, PresaleError::ZeroTokenAmount);
    require!(
        deposit_amount >= presale.buyer_minimum_deposit_cap
            && deposit_amount <= presale.buyer_maximum_deposit_cap,
        PresaleError::DepositAmountOutOfCap
    );

    // 3. Update presale and escrow state
    let deposit_fee_included_amount = calculate_deposit_fee_included_amount_with_max_cap(
        deposit_amount,
        presale.deposit_fee_bps,
        presale.max_deposit_fee,
    )?;

    let deposit_fee = deposit_fee_included_amount
        .checked_sub(deposit_amount)
        .unwrap();

    presale.deposit(&mut escrow, deposit_fee_included_amount, deposit_fee)?;
    presale_handler.end_presale_if_max_cap_reached(&mut presale, current_timestamp)?;

    // 4. Transfer
    let include_transfer_fee_deposit_amount = calculate_transfer_fee_included_amount(
        &ctx.accounts.quote_mint,
        deposit_fee_included_amount,
    )?
    .amount;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.payer_quote_token.to_account_info(),
                to: ctx.accounts.quote_token_vault.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.quote_mint.to_account_info(),
            },
        ),
        include_transfer_fee_deposit_amount,
        ctx.accounts.quote_mint.decimals,
    )?;

    emit_cpi!(EvtDeposit {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        deposit_amount,
        deposit_fee,
        escrow_total_deposit_amount: escrow.total_deposit,
        escrow_total_deposit_fee: escrow.deposit_fee,
        presale_total_deposit_amount: presale.total_deposit,
        presale_total_deposit_fee: presale.total_deposit_fee,
        owner: ctx.accounts.payer.key()
    });

    Ok(())
}
