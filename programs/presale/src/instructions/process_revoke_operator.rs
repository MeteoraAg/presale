use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct RevokeOperatorCtx<'info> {
    #[account(
        mut,
        has_one = creator,
        close = creator,
    )]
    pub operator: AccountLoader<'info, Operator>,

    #[account(mut)]
    pub creator: Signer<'info>,
}
