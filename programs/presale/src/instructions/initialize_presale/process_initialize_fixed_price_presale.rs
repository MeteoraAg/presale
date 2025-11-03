use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InitializeFixedPricePresaleArgs {
    pub common_args: CommonPresaleArgs,
    pub q_price: u128,
    pub disable_withdraw: u8,
    pub disable_earlier_presale_end_once_cap_reached: u8,
    pub padding: [u8; 32],
}

impl InitializeFixedPricePresaleArgs {
    pub fn validate(&self) -> Result<()> {
        self.common_args.validate()?;

        require!(self.q_price > 0, PresaleError::InvalidTokenPrice);

        let disable_withdraw = Bool::try_from(self.disable_withdraw);
        require!(disable_withdraw.is_ok(), PresaleError::InvalidType);

        let disable_earlier_presale_end_once_cap_reached =
            Bool::try_from(self.disable_earlier_presale_end_once_cap_reached);
        require!(
            disable_earlier_presale_end_once_cap_reached.is_ok(),
            PresaleError::InvalidType
        );

        Ok(())
    }
}

pub fn handle_initialize_fixed_price_presale<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
    args: InitializeFixedPricePresaleArgs,
    remaining_account_info: RemainingAccountsInfo,
) -> Result<()> {
    // 1. Validate params
    args.validate()?;

    let InitializeFixedPricePresaleArgs {
        common_args,
        q_price,
        disable_withdraw,
        disable_earlier_presale_end_once_cap_reached,
        ..
    } = args;

    // Safe to convert after validation
    let disable_withdraw: bool = Bool::from(disable_withdraw).into();
    let disable_earlier_presale_end_once_cap_reached: bool =
        Bool::from(disable_earlier_presale_end_once_cap_reached).into();

    let args = HandleInitializePresaleArgs {
        common_args: &common_args,
        presale_mode: PresaleMode::FixedPrice,
        disable_earlier_presale_end_once_cap_reached,
        q_price,
        disable_withdraw,
    };

    handle_initialize_presale(&ctx, args, remaining_account_info)?;

    emit_cpi!(EvtFixedPricePresaleVaultCreate {
        base_mint: ctx.accounts.presale_mint.key(),
        quote_mint: ctx.accounts.quote_token_mint.key(),
        q_price,
        disable_earlier_presale_end_once_cap_reached: disable_earlier_presale_end_once_cap_reached
            .into(),
        disable_withdraw: disable_withdraw.into(),
        args: common_args
    });

    Ok(())
}
