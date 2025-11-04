use crate::*;

pub struct ProcessCreatePresaleVaultArgs<'a, 'c: 'info, 'd, 'e, 'info> {
    pub presale: &'a AccountLoader<'info, Presale>,
    pub presale_params: &'d PresaleArgs,
    pub presale_registries: &'d [PresaleRegistryArgs],
    pub locked_vesting_params: Option<&'d LockedVestingArgs>,
    pub remaining_accounts: &'e mut &'c [AccountInfo<'info>],
    pub mint_pubkeys: InitializePresaleVaultAccountPubkeys,
}

pub fn process_create_presale_vault(params: ProcessCreatePresaleVaultArgs) -> Result<()> {
    let ProcessCreatePresaleVaultArgs {
        presale,
        presale_params,
        presale_registries,
        locked_vesting_params,
        remaining_accounts,
        mint_pubkeys,
    } = params;

    let mut presale_state = presale.load_init()?;
    let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;

    // 1. Initialize presale common fields
    let InitializePresaleVaultAccountPubkeys {
        base_mint,
        quote_mint,
        base_token_vault,
        quote_token_vault,
        owner,
        base,
        base_token_program,
        quote_token_program,
    } = mint_pubkeys;

    presale_state.initialize(PresaleInitializeArgs {
        presale_params,
        locked_vesting_params: locked_vesting_params.cloned(),
        presale_registries,
        base_mint,
        quote_mint,
        base_token_vault,
        quote_token_vault,
        owner,
        current_timestamp,
        base,
        base_token_program,
        quote_token_program,
    })?;

    // 2. Initialize presale mode specific fields
    let presale_handler = get_presale_mode_handler(&presale_state)?;
    presale_handler.initialize_presale(
        presale.key(),
        &mut presale_state,
        presale_params,
        remaining_accounts,
    )?;

    Ok(())
}
