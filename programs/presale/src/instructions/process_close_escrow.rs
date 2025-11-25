use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CloseEscrowCtx<'info> {
    #[account(mut)]
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        mut,
        has_one = presale,
        has_one = owner,
        close = rent_receiver
    )]
    pub escrow: AccountLoader<'info, Escrow>,

    pub owner: Signer<'info>,

    /// CHECK: Account to receive the remaining rent after closing the escrow.
    #[account(mut)]
    pub rent_receiver: UncheckedAccount<'info>,
}

pub fn handle_close_escrow(ctx: Context<CloseEscrowCtx>) -> Result<()> {
    let mut presale = ctx.accounts.presale.load_mut()?;
    let escrow = ctx.accounts.escrow.load()?;

    let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;
    let presale_progress = presale.get_presale_progress(current_timestamp);

    match presale_progress {
        PresaleProgress::Ongoing => {
            ensure_escrow_no_deposit_and_potential_refundable(&escrow)?;
        }
        PresaleProgress::Failed => {
            ensure_escrow_withdrawn_remaining_quote(&escrow)?;
        }
        PresaleProgress::Completed => {
            ensure_escrow_done_claim_and_withdraw_remaining_quote(
                &presale,
                &escrow,
                current_timestamp,
            )?;
        }
        _ => {
            unreachable!(
                "Invalid presale progress state for closing escrow: {:?}",
                presale_progress
            );
        }
    }

    presale.decrease_escrow_count(escrow.registry_index)?;

    emit_cpi!(EvtEscrowClose {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        owner: ctx.accounts.owner.key(),
        rent_receiver: ctx.accounts.rent_receiver.key(),
    });

    Ok(())
}

fn ensure_escrow_done_claim_and_withdraw_remaining_quote(
    presale: &Presale,
    escrow: &Escrow,
    current_timestamp: u64,
) -> Result<()> {
    // 1. Ensure the escrow has withdrawn remaining quote token
    match presale.validate_and_get_escrow_remaining_quote(escrow, current_timestamp) {
        Ok(EscrowRemainingQuoteResult {
            refund_deposit_amount,
            refund_fee_amount,
        }) => {
            if refund_deposit_amount > 0 || refund_fee_amount > 0 {
                require!(
                    escrow.is_remaining_quote_withdrawn(),
                    PresaleError::EscrowNotEmpty
                );
            }
        }
        Err(e) => {
            // If the presale mode does not allow withdraw remaining quote, no need to check for escrow remaining quote is withdrawn or not
            // Bubble up the error if something else is wrong
            if e.ne(&Error::from(
                PresaleError::PresaleNotOpenForWithdrawRemainingQuote,
            )) {
                return Err(e);
            }
        }
    }

    // 2. Ensure the escrow has claimed all bought tokens
    let presale_handler = get_presale_mode_handler(&presale)?;

    let vesting_end_time = presale.vesting_start_time.safe_add(presale.vest_duration)?;

    // Get total dripped bought token at vesting end time
    let escrow_total_claimable_amount: u64 =
        presale_handler.get_escrow_cumulative_claimable_token(presale, escrow, vesting_end_time)?;

    require!(
        escrow.total_claimed_token == escrow_total_claimable_amount,
        PresaleError::EscrowNotEmpty
    );

    Ok(())
}

fn ensure_escrow_no_deposit_and_potential_refundable(escrow: &Escrow) -> Result<()> {
    // We allow closing of escrow on PresaleProgress::Ongoing only when it have no potential refundable amount and deposit
    // If there's any, the user must wait until the presale state is concluded (either Completed or Failed)
    require!(
        escrow.total_deposit == 0 && escrow.total_deposit_fee == 0,
        PresaleError::EscrowNotEmpty
    );
    Ok(())
}

fn ensure_escrow_withdrawn_remaining_quote(escrow: &Escrow) -> Result<()> {
    require!(
        escrow.is_remaining_quote_withdrawn()
            || (escrow.total_deposit == 0 && escrow.total_deposit_fee == 0),
        PresaleError::EscrowNotEmpty
    );
    Ok(())
}
