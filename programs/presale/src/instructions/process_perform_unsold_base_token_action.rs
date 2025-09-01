use crate::*;
use anchor_spl::{
    memo::Memo,
    token_2022::{burn, Burn},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[event_cpi]
#[derive(Accounts)]
pub struct PerformUnsoldBaseTokenActionCtx<'info> {
    #[account(
        mut,
        has_one = base_token_vault,
        has_one = base_mint
    )]
    pub presale: AccountLoader<'info, Presale>,

    #[account(mut)]
    pub base_token_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub base_mint: InterfaceAccount<'info, Mint>,

    /// CHECK: The event authority is derived from the presale program ID
    #[account(
        address = crate::presale_authority::ID,
    )]
    pub presale_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::authority = presale.load()?.owner,
        associated_token::mint = base_mint,
        associated_token::token_program = token_program
    )]
    pub creator_base_token: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,

    pub memo_program: Program<'info, Memo>,
}

pub fn handle_perform_unsold_base_token_action<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, PerformUnsoldBaseTokenActionCtx<'info>>,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;

    let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;
    let presale_progress = presale.get_presale_progress(current_timestamp);

    // 1. Ensure the presale is completed
    require!(
        presale_progress == PresaleProgress::Completed,
        PresaleError::PresaleNotCompleted
    );

    require!(
        !presale.is_unsold_price_token_action_performed(),
        PresaleError::UnsoldBaseTokenActionAlreadyPerformed
    );

    // 2. Compute the total unsold base tokens
    let presale_mode = PresaleMode::from(presale.presale_mode);
    let presale_handler = get_presale_mode_handler(presale_mode);

    let total_token_unsold = presale.get_total_unsold_token(presale_handler.as_ref())?;
    require!(total_token_unsold > 0, PresaleError::NoUnsoldBaseTokens);

    presale.set_unsold_token_action_performed()?;

    // 3. Burn or refund the unsold base tokens to the creator
    let unsold_base_token_action = UnsoldTokenAction::from(presale.unsold_token_action);

    let signer_seeds = &[&presale_authority_seeds!()[..]];

    match unsold_base_token_action {
        UnsoldTokenAction::Burn => burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.base_mint.to_account_info(),
                    from: ctx.accounts.base_token_vault.to_account_info(),
                    authority: ctx.accounts.presale_authority.to_account_info(),
                },
                signer_seeds,
            ),
            total_token_unsold,
        )?,
        UnsoldTokenAction::Refund => {
            let transfer_hook_account = parse_remaining_accounts_for_transfer_hook(
                &mut &ctx.remaining_accounts[..],
                &remaining_accounts_info.slices,
                &[AccountsType::TransferHookBase],
            )?;

            transfer_from_presale_to_user(
                &ctx.accounts.presale_authority,
                &ctx.accounts.base_mint,
                &ctx.accounts.base_token_vault,
                &ctx.accounts.creator_base_token,
                &ctx.accounts.token_program,
                total_token_unsold,
                Some(MemoTransferContext {
                    memo_program: &ctx.accounts.memo_program,
                    memo: PRESALE_MEMO,
                }),
                transfer_hook_account.transfer_hook_base,
            )?;
        }
    }

    emit_cpi!(EvtPerformUnsoldBaseTokenAction {
        presale: ctx.accounts.presale.key(),
        unsold_base_token_action: unsold_base_token_action.into(),
        total_token_unsold,
    });

    Ok(())
}
