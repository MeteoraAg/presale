use anchor_spl::{
    memo::Memo,
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
    pub owner_base_token: Box<InterfaceAccount<'info, TokenAccount>>,

    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,

    pub memo_program: Program<'info, Memo>,
}

pub fn handle_claim<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, ClaimCtx<'info>>,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let mut escrow = ctx.accounts.escrow.load_mut()?;

    // 1. Ensure the presale is in a state that allows claiming
    let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;
    let presale_progress = presale.get_presale_progress(current_timestamp);

    require!(
        presale_progress == PresaleProgress::Completed,
        PresaleError::PresaleNotOpenForClaim
    );

    // 2. Process claim
    let presale_mode = PresaleMode::from(presale.presale_mode);
    let presale_handler = get_presale_mode_handler(presale_mode);

    presale_handler.update_pending_claim_amount(&presale, &mut escrow, current_timestamp)?;

    let pending_claim_token = escrow.pending_claim_token;

    if pending_claim_token > 0 {
        presale.claim(&mut escrow)?;

        let transfer_hook_accounts = parse_remaining_accounts_for_transfer_hook(
            &mut &ctx.remaining_accounts[..],
            &remaining_accounts_info.slices,
            &[AccountsType::TransferHookBase],
        )?;

        transfer_from_presale_to_user(
            &ctx.accounts.presale_authority,
            &ctx.accounts.base_mint,
            &ctx.accounts.base_token_vault,
            &ctx.accounts.owner_base_token,
            &ctx.accounts.token_program,
            pending_claim_token,
            Some(MemoTransferContext {
                memo_program: &ctx.accounts.memo_program,
                memo: PRESALE_MEMO,
            }),
            transfer_hook_accounts.transfer_hook_base,
        )?;
    }

    let excluded_fee_claim_amount =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.base_mint, pending_claim_token)?
            .amount;

    emit_cpi!(EvtClaim {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        owner: ctx.accounts.owner.key(),
        claim_amount: excluded_fee_claim_amount,
        escrow_total_claim_amount: escrow.total_claimed_token,
        presale_total_claim_amount: presale.total_claimed_token
    });

    Ok(())
}
