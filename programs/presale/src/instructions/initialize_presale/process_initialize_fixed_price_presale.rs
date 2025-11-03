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
    };

    // 2. Initialize common presale fields
    handle_initialize_presale_common_fields(&ctx, args, remaining_account_info)?;

    // 3. Validate presale mode specific fields
    let whitelist_mode = WhitelistMode::from(common_args.presale_params.whitelist_mode);
    let presale_registries = &common_args.presale_registries;
    let presale_params = &common_args.presale_params;

    for registry in presale_registries {
        if !registry.is_uninitialized() {
            // ensure buyer_minimum_deposit_cap and buyer_maximum_deposit_cap can buy at least 1 token and not exceed u64::MAX token
            ensure_token_buyable(q_price, registry.buyer_minimum_deposit_cap)?;
            ensure_token_buyable(q_price, registry.buyer_maximum_deposit_cap)?;

            // In permissioned whitelist mode, ensure buyer min/max cap is set to minimum and maximum allowed range
            // This reduces the mistake of setting unusable buyer cap in permissioned presale at offchain
            if whitelist_mode.is_permissioned() {
                let min_quote_amount = calculate_min_quote_amount_for_base_lamport(q_price)?;

                require!(
                    registry.buyer_minimum_deposit_cap == min_quote_amount,
                    PresaleError::InvalidBuyerCapRange
                );

                require!(
                    registry.buyer_maximum_deposit_cap == presale_params.presale_maximum_cap,
                    PresaleError::InvalidBuyerCapRange
                );
            }
        }
    }

    let mut presale = ctx.accounts.presale.load_init()?;
    // Ensure presale supply is enough to fulfill presale maximum cap
    ensure_enough_presale_supply(
        q_price,
        presale.presale_supply,
        presale_params.presale_maximum_cap,
    )?;

    // 4. Initialize fixed price presale specific fields
    FixedPricePresaleHandler::initialize(
        &mut presale.presale_mode_raw_data,
        q_price,
        disable_withdraw,
        disable_earlier_presale_end_once_cap_reached,
    )?;

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

pub fn ensure_token_buyable(q_price: u128, amount: u64) -> Result<()> {
    let max_token_bought = calculate_token_bought(q_price, amount)?;

    require!(max_token_bought > 0, PresaleError::ZeroTokenAmount);
    require!(
        max_token_bought <= u64::MAX as u128,
        PresaleError::InvalidTokenPrice
    );
    Ok(())
}

fn ensure_enough_presale_supply(
    q_price: u128,
    presale_supply: u64,
    maximum_cap: u64,
) -> Result<()> {
    let max_presale_supply_bought = calculate_token_bought(q_price, maximum_cap)?;

    require!(
        max_presale_supply_bought <= u128::from(presale_supply),
        PresaleError::InvalidTokenPrice
    );
    Ok(())
}
