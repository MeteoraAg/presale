use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

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

// Max amount doesn't include the transfer fees. This is the maximum amount the user wants to deposit.
pub fn handle_deposit<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, DepositCtx<'info>>,
    max_amount: u64,
    remaining_account_info: RemainingAccountsInfo,
) -> Result<()> {
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

    // TODO: Should we ensure that the total deposit amount can buy at least one token? Because during init presale we only validate the max buyer cap.
    require!(deposit_amount > 0, PresaleError::ZeroTokenAmount);
    require!(
        deposit_amount >= presale.buyer_minimum_deposit_cap
            && deposit_amount <= presale.buyer_maximum_deposit_cap,
        PresaleError::DepositAmountOutOfCap
    );

    // 3. Update presale and escrow state
    presale.deposit(&mut escrow, deposit_amount)?;
    presale_handler.end_presale_if_max_cap_reached(&mut presale, current_timestamp)?;

    // 4. Transfer
    let include_transfer_fee_deposit_amount =
        calculate_transfer_fee_included_amount(&ctx.accounts.quote_mint, deposit_amount)?.amount;

    let transfer_hook_accounts = parse_remaining_accounts_for_transfer_hook(
        &mut &ctx.remaining_accounts[..],
        &remaining_account_info.slices,
        &[AccountsType::TransferHookQuote],
    )?;

    transfer_from_user(
        &ctx.accounts.payer,
        &ctx.accounts.quote_mint,
        &ctx.accounts.payer_quote_token,
        &ctx.accounts.quote_token_vault,
        &ctx.accounts.token_program,
        include_transfer_fee_deposit_amount,
        None,
        transfer_hook_accounts.transfer_hook_quote,
    )?;

    emit_cpi!(EvtDeposit {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        deposit_amount,
        escrow_total_deposit_amount: escrow.total_deposit,
        presale_total_deposit_amount: presale.total_deposit,
        owner: ctx.accounts.payer.key()
    });

    Ok(())
}
