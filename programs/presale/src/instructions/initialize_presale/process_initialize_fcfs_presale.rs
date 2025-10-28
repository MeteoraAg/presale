use crate::*;

#[derive(AnchorSerialize, AnchorDeserialize, Default)]
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
        disable_earlier_presale_end_once_cap_reached,
        q_price: 0,
        disable_withdraw: true,
    };

    handle_initialize_presale(ctx, args, remaining_account_info)
}
