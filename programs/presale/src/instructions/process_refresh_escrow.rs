use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct RefreshEscrowCtx<'info> {
    pub presale: AccountLoader<'info, Presale>,

    #[account(
        mut,
        has_one = presale,
    )]
    pub escrow: AccountLoader<'info, Escrow>,
}

pub fn handle_refresh_escrow(ctx: Context<RefreshEscrowCtx>) -> Result<()> {
    let presale = ctx.accounts.presale.load()?;
    let mut escrow = ctx.accounts.escrow.load_mut()?;

    let current_timestamp: u64 = Clock::get()?.unix_timestamp as u64;

    if current_timestamp >= presale.vesting_start_time {
        let presale_mode = PresaleMode::from(presale.presale_mode);
        let presale_handler = get_presale_mode_handler(presale_mode);

        presale_handler.update_pending_claim_amount(&presale, &mut escrow, current_timestamp)?;
    }

    emit_cpi!(EvtEscrowRefresh {
        presale: ctx.accounts.presale.key(),
        escrow: ctx.accounts.escrow.key(),
        owner: escrow.owner,
        pending_claim_token: escrow.pending_claim_token,
        current_timestamp,
    });

    Ok(())
}
