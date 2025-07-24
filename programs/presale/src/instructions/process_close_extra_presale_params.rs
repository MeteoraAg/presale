use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CloseFixedPricePresaleArgsCtx {
    #[account(
        mut,
        close = owner,
        has_one = owner,
    )]
    pub fixed_price_presale_args: AccountLoader<'info, FixedPricePresaleExtraArgs>,

    #[account(mut)]
    pub owner: Signer<'info>,
}

pub fn handle_close_fixed_price_presale_args(
    ctx: Context<CloseFixedPricePresaleArgsCtx>,
) -> Result<()> {
    let fixed_price_presale_args = ctx.accounts.fixed_price_presale_args.load()?;

    emit_cpi!(EvtFixedPricePresaleArgsClose {
        presale: fixed_price_presale_args.presale,
        fixed_price_presale_args: ctx.accounts.fixed_price_presale_args.key(),
        owner: ctx.accounts.owner.key(),
    });
    Ok(())
}
