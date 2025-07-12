use anchor_spl::{
    token_2022::{transfer_checked, TransferChecked},
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

    #[account(mut, 
        has_one = presale, 
        has_one = owner
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    pub owner_quote_token: Box<InterfaceAccount<'info, TokenAccount>>,
    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_withdraw(ctx: Context<WithdrawCtx>, amount: u64) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let mut escrow = ctx.accounts.escrow.load_mut()?;

    // 1. Ensure presale is ongoing
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
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
    let amount_withdrawn =
        presale_mode_handler.process_withdraw(&mut presale, &mut escrow, amount)?;

    let exclude_transfer_fee_amount_withdrawn =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.quote_mint, amount_withdrawn)?.amount;

    let signer_seeds = &[&presale_authority_seeds!()[..]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.quote_token_vault.to_account_info(),
                mint: ctx.accounts.quote_mint.to_account_info(),
                to: ctx.accounts.owner_quote_token.to_account_info(),
                authority: ctx.accounts.presale_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount_withdrawn,
        ctx.accounts.quote_mint.decimals,
    )?;

    emit_cpi!(EvtWithdraw {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        owner: ctx.accounts.owner.key(),
        withdraw_amount: exclude_transfer_fee_amount_withdrawn,
        escrow_total_deposit_amount: escrow.total_deposit,
        escrow_total_deposit_fee: escrow.deposit_fee,
        presale_total_deposit_amount: presale.total_deposit,
        presale_total_deposit_fee: presale.total_deposit_fee
    });

    Ok(())
}
