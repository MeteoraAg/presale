use anchor_spl::{
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimCtx<'info> {
    #[account(
        mut,
        has_one = base_token_vault,
        has_one = base_mint,
    )]
    pub presale: AccountLoader<'info, Presale>,

    #[account(mut)]
    pub base_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub base_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: The presale authority
    pub presale_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        has_one = owner
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    pub owner_base_token: Box<InterfaceAccount<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_claim(ctx: Context<ClaimCtx>) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let mut escrow = ctx.accounts.escrow.load_mut()?;

    // 1. Ensure the presale is in a state that allows claiming
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let presale_progress = presale.get_presale_progress(current_timestamp);

    require!(
        presale_progress == PresaleProgress::Completed,
        PresaleError::PresaleNotOpenForClaim
    );

    require!(
        !presale.in_locking_period(current_timestamp),
        PresaleError::PresaleNotOpenForClaim
    );

    // 2. Process claim
    let presale_mode = PresaleMode::from(presale.presale_mode);
    let presale_handler = get_presale_mode_handler(presale_mode);

    let claim_amount =
        presale_handler.process_claim(&mut presale, &mut escrow, current_timestamp)?;

    require!(claim_amount > 0, PresaleError::ZeroTokenAmount);

    let signer_seeds = &[&presale_authority_seeds!()[..]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.base_token_vault.to_account_info(),
                to: ctx.accounts.owner_base_token.to_account_info(),
                mint: ctx.accounts.base_mint.to_account_info(),
                authority: ctx.accounts.presale_authority.to_account_info(),
            },
            signer_seeds,
        ),
        claim_amount,
        ctx.accounts.base_mint.decimals,
    )?;

    emit_cpi!(EvtClaim {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        owner: ctx.accounts.owner.key(),
        claim_amount,
        escrow_total_claim_amount: escrow.total_claimed_token,
        presale_total_claim_amount: presale.total_claimed_token
    });

    Ok(())
}
