use crate::*;
use anchor_spl::{
    memo::Memo,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

#[event_cpi]
#[derive(Accounts)]
pub struct CreatorCollectFeeCtx<'info> {
    #[account(
        mut,
        has_one = owner,
        has_one = quote_mint,
        has_one = quote_token_vault
    )]
    pub presale: AccountLoader<'info, Presale>,

    /// CHECK: presale_authority
    #[account(
       address = presale_authority::ID
    )]
    pub presale_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub quote_token_vault: InterfaceAccount<'info, TokenAccount>,

    pub quote_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub fee_receiving_account: InterfaceAccount<'info, TokenAccount>,

    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub memo_program: Program<'info, Memo>,
}

pub fn handle_creator_collect_fee<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, CreatorCollectFeeCtx<'info>>,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let current_timestamp = Clock::get()?.unix_timestamp;

    // 1. Validate is deposit fee allowed or not
    require!(
        ensure_allow_collect_deposit_fee(&presale, current_timestamp.safe_cast()?),
        PresaleError::PresaleNotOpenForCollectFee
    );

    // 2. Collect fee
    let total_deposit_fee = presale.get_total_collected_fee()?;

    // 3. Mark deposit fee as collected
    presale.set_deposit_fee_collected();

    if total_deposit_fee > 0 {
        let transfer_hook_accounts = parse_remaining_accounts_for_transfer_hook(
            &mut &ctx.remaining_accounts[..],
            &remaining_accounts_info.slices,
            &[AccountsType::TransferHookQuote],
        )?;

        transfer_from_presale_to_user(
            &ctx.accounts.presale_authority,
            &ctx.accounts.quote_mint,
            &ctx.accounts.quote_token_vault,
            &ctx.accounts.fee_receiving_account,
            &ctx.accounts.token_program,
            total_deposit_fee,
            Some(MemoTransferContext {
                memo_program: &ctx.accounts.memo_program,
                memo: PRESALE_MEMO,
            }),
            transfer_hook_accounts.transfer_hook_quote,
        )?;
    }

    let transfer_fee_excluded_deposit_fee =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.quote_mint, total_deposit_fee)?.amount;

    emit_cpi!(EvtCreatorCollectFee {
        presale: ctx.accounts.presale.key(),
        owner: ctx.accounts.owner.key(),
        total_collected_fee: transfer_fee_excluded_deposit_fee,
    });

    Ok(())
}

fn ensure_allow_collect_deposit_fee(presale: &Presale, current_timestamp: u64) -> bool {
    let presale_progress = presale.get_presale_progress(current_timestamp);
    presale_progress == PresaleProgress::Completed && !presale.is_deposit_fee_collected()
}
