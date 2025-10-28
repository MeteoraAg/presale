use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Default)]
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
        // For prorata presale, we do not allow disable earlier presale end once cap is reached
        disable_earlier_presale_end_once_cap_reached: false,
        // Prorata is dynamic price, so q_price is 0
        q_price: 0,
        // For prorata presale, we always allow withdraw
        disable_withdraw: false,
    };

    handle_initialize_presale(ctx, args, remaining_account_info)
}
