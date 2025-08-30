use anchor_spl::{
    memo::Memo,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawCtx {
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
        address = crate::const_pda::presale_authority::ID,
    )]
    /// CHECK: The presale authority is the PDA of the presale.
    pub presale_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        has_one = presale,
        has_one = owner
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    #[account(mut)]
    pub owner_quote_token: Box<InterfaceAccount<'info, TokenAccount>>,
    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub memo_program: Program<'info, Memo>,
}

pub fn handle_withdraw<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, WithdrawCtx<'info>>,
    amount: u64,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let mut escrow = ctx.accounts.escrow.load_mut()?;

    // 1. Ensure presale is ongoing
    let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;
    let presale_progress = presale.get_presale_progress(current_timestamp);
    require!(
        presale_progress == PresaleProgress::Ongoing,
        PresaleError::PresaleNotOpenForWithdraw
    );

    // 2. Ensure withdraw amount > 0
    require!(amount > 0, PresaleError::ZeroTokenAmount);

    // 3. Have enough balance to withdraw
    require!(
        escrow.total_deposit >= amount,
        PresaleError::InsufficientEscrowBalance
    );

    // 4. Ensure presale mode allows withdraw
    let presale_mode = PresaleMode::from(presale.presale_mode);
    let presale_mode_handler = get_presale_mode_handler(presale_mode);
    require!(
        presale_mode_handler.can_withdraw(),
        PresaleError::PresaleNotOpenForWithdraw
    );

    // 5. Update escrow and presale state
    presale_mode_handler.process_withdraw(&mut presale, &mut escrow, amount)?;

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
        amount,
        Some(MemoTransferContext {
            memo_program: &ctx.accounts.memo_program,
            memo: PRESALE_MEMO,
        }),
        transfer_hook_accounts.transfer_hook_quote,
    )?;

    let exclude_transfer_fee_amount_withdrawn =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.quote_mint, amount)?.amount;

    // TODO: Should we ensure that the total deposit amount can buy at least one token after withdraw?

    emit_cpi!(EvtWithdraw {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        owner: ctx.accounts.owner.key(),
        withdraw_amount: exclude_transfer_fee_amount_withdrawn,
        escrow_total_deposit_amount: escrow.total_deposit,
        presale_total_deposit_amount: presale.total_deposit,
    });

    Ok(())
}
