use crate::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CreateOperatorCtx<'info> {
    #[account(
        init,
        seeds = [
            crate::constants::seeds::OPERATOR_PREFIX,
            creator.key().as_ref(),
            operator.key().as_ref()
        ],
        space = 8 + Operator::INIT_SPACE,
        payer = creator,
        bump
    )]
    pub operator: AccountLoader<'info, Operator>,

    /// CHECK: Owner of the operator account
    pub operator_owner: UncheckedAccount<'info>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_operator(ctx: Context<CreateOperatorCtx>) -> Result<()> {
    let operator = &mut ctx.accounts.operator.load_init()?;

    // TODO: Should further scope down by including presale pubkey for higher security?
    operator.initialize(
        ctx.accounts.operator_owner.key(),
        ctx.accounts.creator.key(),
    );

    emit_cpi!(EvtOperatorCreate {
        creator: ctx.accounts.creator.key(),
        operator: ctx.accounts.operator.key(),
        operator_owner: ctx.accounts.operator_owner.key()
    });

    Ok(())
}
