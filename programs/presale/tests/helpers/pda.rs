use anchor_client::solana_sdk::pubkey::Pubkey;

pub fn derive_presale(mint: &Pubkey, quote: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            presale::seeds::PRESALE_PREFIX.as_ref(),
            mint.as_ref(),
            quote.as_ref(),
        ],
        program_id,
    )
    .0
}

pub fn derive_presale_vault(presale: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[presale::seeds::BASE_VAULT_PREFIX.as_ref(), presale.as_ref()],
        program_id,
    )
    .0
}

pub fn derive_quote_vault(presale: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            presale::seeds::QUOTE_VAULT_PREFIX.as_ref(),
            presale.as_ref(),
        ],
        program_id,
    )
    .0
}

pub fn derive_event_authority(program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], program_id).0
}

pub fn derive_fixed_price_presale_args(
    mint: &Pubkey,
    quote: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    let presale = derive_presale(mint, quote, program_id);
    Pubkey::find_program_address(
        &[
            presale::seeds::FIXED_PRICE_PRESALE_PARAM_PREFIX.as_ref(),
            presale.as_ref(),
        ],
        program_id,
    )
    .0
}
