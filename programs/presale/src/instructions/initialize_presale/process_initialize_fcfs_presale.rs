use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InitializeFcfsPresaleArgs {
    pub common_args: CommonPresaleArgs,
    pub disable_earlier_presale_end_once_cap_reached: u8,
    pub padding: [u8; 32],
}

impl InitializeFcfsPresaleArgs {
    pub fn validate(&self) -> Result<()> {
        self.common_args.validate()?;

        let maybe_disable_earlier_presale_end_once_cap_reached =
            Bool::try_from(self.disable_earlier_presale_end_once_cap_reached);
        require!(
            maybe_disable_earlier_presale_end_once_cap_reached.is_ok(),
            PresaleError::InvalidType
        );

        Ok(())
    }
}

pub fn handle_initialize_fcfs_presale<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
    args: InitializeFcfsPresaleArgs,
    remaining_account_info: RemainingAccountsInfo,
) -> Result<()> {
    // 1. Validate params
    args.validate()?;

    let InitializeFcfsPresaleArgs {
        common_args,
        disable_earlier_presale_end_once_cap_reached,
        ..
    } = args;

    // Safe to convert after validation
    let disable_earlier_presale_end_once_cap_reached: bool =
        Bool::from(disable_earlier_presale_end_once_cap_reached).into();

    let args = HandleInitializePresaleArgs {
        common_args: &common_args,
        presale_mode: PresaleMode::Fcfs,
    };

    handle_initialize_presale_common_fields(&ctx, args, remaining_account_info)?;

    // Disc is written only after AccountsExit::exit is called, so this is safe.
    // 2. Validate presale mode specific fields
    let presale_params = &common_args.presale_params;
    let presale_registries = common_args.presale_registries.as_ref();

    let whitelist_mode = WhitelistMode::from(presale_params.whitelist_mode);

    if whitelist_mode.is_permissioned() {
        enforce_dynamic_price_registries_max_buyer_cap_range(presale_params, &presale_registries)?;
    }

    // 3. Initialize FCFS presale specific fields
    let mut presale = ctx.accounts.presale.load_init()?;

    FcfsPresaleHandler::initialize(
        &mut presale.presale_mode_raw_data,
        disable_earlier_presale_end_once_cap_reached,
    )?;

    emit_cpi!(EvtFcfsPresaleVaultCreate {
        base_mint: ctx.accounts.presale_mint.key(),
        quote_mint: ctx.accounts.quote_token_mint.key(),
        args: common_args,
        disable_earlier_presale_end_once_cap_reached: disable_earlier_presale_end_once_cap_reached
            .into(),
    });

    Ok(())
}
