use crate::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(params: InitializeFixedPricePresaleExtraArgs)]
pub struct InitializeFixedPricePresaleArgsCtx {
    #[account(
        init,
        seeds = [
            crate::constants::seeds::FIXED_PRICE_PRESALE_PARAM_PREFIX.as_ref(),
            params.presale.as_ref(),
        ],
        payer = payer,
        bump,
        space = 8 + FixedPricePresaleExtraArgs::INIT_SPACE
    )]
    pub fixed_price_presale_params: AccountLoader<'info, FixedPricePresaleExtraArgs>,

    /// CHECK: owner
    pub owner: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Default)]
pub struct InitializeFixedPricePresaleExtraArgs {
    pub presale: Pubkey,
    pub padding0: u8,
    pub q_price: u128,
    pub padding1: [u64; 8],
}

pub fn handle_initialize_fixed_price_presale_args(
    ctx: Context<InitializeFixedPricePresaleArgsCtx>,
    params: InitializeFixedPricePresaleExtraArgs,
) -> Result<()> {
    let InitializeFixedPricePresaleExtraArgs {
        presale, q_price, ..
    } = params;

    let fixed_price_presale_params = &mut ctx.accounts.fixed_price_presale_params.load_init()?;
    fixed_price_presale_params.validate_and_initialize(
        q_price,
        ctx.accounts.owner.key(),
        presale,
    )?;

    emit_cpi!(EvtFixedPricePresaleArgsCreate { presale, q_price });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_initialize_fixed_price_presale_extra_args_size() {
        let args = InitializeFixedPricePresaleExtraArgs::default();
        assert_eq!(args.try_to_vec().unwrap().len(), 113);
    }
}
