use anchor_spl::{
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CreatorWithdrawCtx<'info> {
    #[account(
        mut,
        has_one = quote_token_vault,
        has_one = quote_mint,
        has_one = owner,
    )]
    pub presale: AccountLoader<'info, Presale>,

    /// CHECK: This is the presale authority
    #[account(
        address = crate::presale_authority::ID,
    )]
    pub presale_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub quote_token_vault: InterfaceAccount<'info, TokenAccount>,
    pub quote_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = quote_mint,
        associated_token::authority = crate::TREASURY_ID
    )]
    pub protocol_fee_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub owner_token: InterfaceAccount<'info, TokenAccount>,
    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_creator_withdraw(ctx: Context<CreatorWithdrawCtx>) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;

    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let presale_progress = presale.get_presale_progress(current_timestamp);

    // 1. Ensure presale is completed
    require!(
        presale_progress == PresaleProgress::Completed,
        PresaleError::PresaleNotCompleted
    );

    // 2. Ensure creator haven't withdrawn yet
    require!(
        !presale.has_creator_withdrawn(),
        PresaleError::CreatorAlreadyWithdrawn
    );

    presale.update_creator_withdrawn()?;

    let protocol_charged_deposit_fee =
        calculate_fee_amount(presale.total_deposit_fee, presale.deposit_fee_bps)?;
    let creator_deposit_fee = presale
        .total_deposit_fee
        .checked_sub(protocol_charged_deposit_fee)
        .unwrap();
    let creator_withdraw_amount = presale
        .total_deposit
        .checked_add(creator_deposit_fee)
        .unwrap();

    let signer_seeds = &[&presale_authority_seeds!()[..]];
    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.quote_token_vault.to_account_info(),
                to: ctx.accounts.owner_token.to_account_info(),
                mint: ctx.accounts.quote_mint.to_account_info(),
                authority: ctx.accounts.presale_authority.to_account_info(),
            },
            signer_seeds,
        ),
        creator_withdraw_amount,
        ctx.accounts.quote_mint.decimals,
    )?;

    transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.quote_token_vault.to_account_info(),
                to: ctx.accounts.protocol_fee_vault.to_account_info(),
                mint: ctx.accounts.quote_mint.to_account_info(),
                authority: ctx.accounts.presale_authority.to_account_info(),
            },
            signer_seeds,
        ),
        protocol_charged_deposit_fee,
        ctx.accounts.quote_mint.decimals,
    )?;

    emit_cpi!(EvtCreatorWithdraw {
        presale: ctx.accounts.presale.key(),
        creator_withdraw_amount,
        protocol_fee_amount: protocol_charged_deposit_fee,
        creator_deposit_fee,
        creator: ctx.accounts.owner.key(),
    });

    Ok(())
}
