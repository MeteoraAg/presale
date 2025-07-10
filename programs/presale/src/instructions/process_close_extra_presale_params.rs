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
