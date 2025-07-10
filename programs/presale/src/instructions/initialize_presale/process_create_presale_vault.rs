use crate::*;

pub struct ProcessCreatePresaleVaultArgs<'a, 'c: 'info, 'd, 'info> {
    pub presale: &'a AccountLoader<'info, Presale>,
    pub tokenomic_params: &'d TokenomicArgs,
    pub presale_params: &'d PresaleArgs,
    pub locked_vesting_params: Option<&'d LockedVestingArgs>,
    pub remaining_accounts: &'c [AccountInfo<'info>],
}

pub fn process_create_presale_vault(params: ProcessCreatePresaleVaultArgs) -> Result<()> {
    let ProcessCreatePresaleVaultArgs {
        presale,
        tokenomic_params,
        presale_params,
        locked_vesting_params,
        remaining_accounts,
    } = params;

    let mut presale = presale.load_init()?;
    let presale_mode = PresaleMode::from(presale_params.presale_mode);

    match presale_mode {
        PresaleMode::FixedPrice => {
            initialize_fixed_price_presale_vault(
                &mut presale,
                tokenomic_params,
                presale_params,
                locked_vesting_params,
                remaining_accounts,
            )?;
        }
        PresaleMode::Prorata | PresaleMode::Fcfs => {
            todo!("Implement Prorata and FCFS presale modes")
        }
    }

    Ok(())
}

fn ensure_token_buyable(
    q_price: u128,
    amount: u64,
    fee_bps: u16,
    max_deposit_fee: u64,
) -> Result<()> {
    let fee_amount = calculate_fee_amount(amount, fee_bps, Rounding::Up)?;
    let fee_amount = fee_amount.min(max_deposit_fee);
    let exclude_fee_amount = amount.checked_sub(fee_amount).unwrap();
    let q_amount = u128::from(exclude_fee_amount).checked_shl(64).unwrap();
    let max_token_bought = q_amount.checked_div(q_price).unwrap();

    require!(max_token_bought > 0, PresaleError::ZeroTokenAmount);
    require!(
        max_token_bought <= u64::MAX as u128,
        PresaleError::InvalidTokenPrice
    );
    Ok(())
}

fn initialize_fixed_price_presale_vault<'c: 'info, 'info>(
    presale: &mut Presale,
    tokenomic_params: &TokenomicArgs,
    presale_params: &PresaleArgs,
    locked_vesting_params: Option<&LockedVestingArgs>,
    remaining_accounts: &'c [AccountInfo<'info>],
) -> Result<()> {
    // 1. Get extra params about fixed price presale mode
    let presale_extra_param_ai = remaining_accounts
        .first()
        .ok_or_else(|| error!(PresaleError::MissingPresaleExtraParams))?;

    let presale_extra_param_al =
        AccountLoader::<FixedPricePresaleExtraArgs>::try_from(presale_extra_param_ai)?;

    let presale_extra_param = presale_extra_param_al.load()?;

    // 2. Validate fixed price presale parameters
    ensure_token_buyable(
        presale_extra_param.q_price,
        presale_params.buyer_maximum_deposit_cap,
        presale_params.deposit_fee_bps,
        presale_params.max_deposit_fee,
    )?;

    let current_timestamp = Clock::get()?.unix_timestamp as u64;

    // 3. Create presale vault
    presale.initialize(
        *tokenomic_params,
        *presale_params,
        locked_vesting_params.cloned(),
        Some(*presale_extra_param),
        current_timestamp,
    );

    Ok(())
}
