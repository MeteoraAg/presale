use anchor_spl::{
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CreatorClaimCtx<'info> {
    #[account(
        mut,
        has_one = owner,
        has_one = base_token_vault,
        has_one = base_mint
    )]
    pub presale: AccountLoader<'info, Presale>,

    /// CHECK: The presale authority must be the PDA of the presale
    #[account(
        address = crate::const_pda::presale_authority::ID,
    )]
    pub presale_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub base_token_vault: InterfaceAccount<'info, TokenAccount>,
    pub base_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub owner_token_account: InterfaceAccount<'info, TokenAccount>,
    pub owner: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_creator_claim(ctx: Context<CreatorClaimCtx>) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;

    // 1. Ensure the presale is open for creator to claim
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    require!(
        current_timestamp >= presale.creator_vest_start_time,
        PresaleError::PresaleNotOpenForClaim
    );

    // 2. Calculate the total unsold tokens
    let presale_mode = PresaleMode::from(presale.presale_mode);
    let presale_handler = get_presale_mode_handler(presale_mode);

    let total_unsold_token = if presale.should_lock_unsold_token() {
        presale.get_total_unsold_token(&presale_handler)?
    } else {
        0
    };

    // 3. Calculate the total base token belongs to the creator
    let creator_total_base_token = presale
        .creator_supply
        .checked_add(total_unsold_token)
        .unwrap();

    // 4. Calculate the total base token to be claimed
    let dripped_base_token = calculate_dripped_amount_for_user(
        presale.creator_vest_start_time,
        presale.creator_vest_duration,
        current_timestamp,
        creator_total_base_token,
        // No need to care for share here, as it's only for creator
        1,
        1,
    )?;

    let dripped_base_token: u64 = dripped_base_token.try_into().unwrap();

    let claim_amount = dripped_base_token
        .checked_sub(presale.total_creator_claimed_token)
        .unwrap();

    // 5. Update the presale state
    presale.update_total_creator_claimed_token(claim_amount)?;

    if claim_amount > 0 {
        let signer_seeds = &[&presale_authority_seeds!()[..]];
        transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.base_token_vault.to_account_info(),
                    to: ctx.accounts.owner_token_account.to_account_info(),
                    authority: ctx.accounts.presale_authority.to_account_info(),
                    mint: ctx.accounts.base_mint.to_account_info(),
                },
                signer_seeds,
            ),
            claim_amount,
            ctx.accounts.base_mint.decimals,
        )?;
    }

    let exclude_fee_claim_amount =
        calculate_transfer_fee_excluded_amount(&ctx.accounts.base_mint, claim_amount)?.amount;

    emit_cpi!(EvtCreatorClaim {
        presale: ctx.accounts.presale.key(),
        creator: ctx.accounts.owner.key(),
        claim_amount: exclude_fee_claim_amount,
        creator_total_claimed_amount: presale.total_creator_claimed_token
    });

    Ok(())
}
