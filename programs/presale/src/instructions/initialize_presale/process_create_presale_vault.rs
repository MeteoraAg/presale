use crate::*;

pub struct ProcessCreatePresaleVaultArgs<'a, 'd, 'e: 'd, 'info> {
    pub presale: &'a AccountLoader<'info, Presale>,
    pub args: &'d HandleInitializePresaleArgs<'e>,
    pub mint_pubkeys: InitializePresaleVaultAccountPubkeys,
}

pub fn process_create_presale_vault(params: ProcessCreatePresaleVaultArgs) -> Result<()> {
    let ProcessCreatePresaleVaultArgs {
        presale,
        mint_pubkeys,
        args,
    } = params;

    let &HandleInitializePresaleArgs {
        presale_mode,
        common_args,
        disable_earlier_presale_end_once_cap_reached,
        disable_withdraw,
        q_price,
    } = args;

    let mut presale_state = presale.load_init()?;
    let presale_handler = get_presale_mode_handler(presale_mode);

    presale_handler.initialize_presale(
        &mut presale_state,
        common_args,
        mint_pubkeys,
        disable_withdraw,
        q_price,
        disable_earlier_presale_end_once_cap_reached,
    )?;

    Ok(())
}
