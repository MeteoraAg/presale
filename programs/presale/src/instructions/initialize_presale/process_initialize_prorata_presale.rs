use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct InitializeProrataPresaleArgs {
    pub common_args: CommonPresaleArgs,
    pub padding: [u8; 32],
}

impl InitializeProrataPresaleArgs {
    pub fn validate(&self) -> Result<()> {
        self.common_args.validate()?;
        Ok(())
    }
}

pub fn handle_initialize_prorata_presale<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, InitializePresaleCtx<'info>>,
    args: InitializeProrataPresaleArgs,
    remaining_account_info: RemainingAccountsInfo,
) -> Result<()> {
    // 1. Validate params
    args.validate()?;

    let InitializeProrataPresaleArgs { common_args, .. } = args;

    let args = HandleInitializePresaleArgs {
        common_args: &common_args,
        presale_mode: PresaleMode::Prorata,
    };

    handle_initialize_presale_common_fields(&ctx, args, remaining_account_info)?;

    // 2. Validate and initialize presale mode specific fields
    let whitelist_mode = WhitelistMode::from(common_args.presale_params.whitelist_mode);

    let presale_params = &common_args.presale_params;
    let presale_registries = common_args.presale_registries.as_ref();

    if whitelist_mode.is_permissioned() {
        enforce_dynamic_price_registries_max_buyer_cap_range(presale_params, presale_registries)?;
    }

    emit_cpi!(EvtProrataPresaleVaultCreate {
        base_mint: ctx.accounts.presale_mint.key(),
        quote_mint: ctx.accounts.quote_token_mint.key(),
        args: common_args
    });

    Ok(())
}
