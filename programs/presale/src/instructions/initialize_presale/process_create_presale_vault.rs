use crate::*;

pub struct ProcessCreatePresaleVaultArgs<'a, 'c: 'info, 'd, 'e, 'info> {
    pub presale: &'a AccountLoader<'info, Presale>,
    pub tokenomic_params: &'d TokenomicArgs,
    pub presale_params: &'d PresaleArgs,
    pub locked_vesting_params: Option<&'d LockedVestingArgs>,
    pub remaining_accounts: &'e mut &'c [AccountInfo<'info>],
    pub mint_pubkeys: InitializePresaleVaultAccountPubkeys,
}

pub fn process_create_presale_vault(params: ProcessCreatePresaleVaultArgs) -> Result<()> {
    let ProcessCreatePresaleVaultArgs {
        presale,
        tokenomic_params,
        presale_params,
        locked_vesting_params,
        remaining_accounts,
        mint_pubkeys,
    } = params;

    let mut presale_state = presale.load_init()?;
    let presale_mode = PresaleMode::from(presale_params.presale_mode);
    let presale_handler = get_presale_mode_handler(presale_mode);

    presale_handler.initialize_presale(
        presale.key(),
        &mut presale_state,
        tokenomic_params,
        presale_params,
        locked_vesting_params,
        mint_pubkeys,
        remaining_accounts,
    )?;

    Ok(())
}
