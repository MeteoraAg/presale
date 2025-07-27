use anchor_client::solana_sdk::pubkey::Pubkey;

pub fn derive_presale(mint: &Pubkey, quote: &Pubkey, base: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            presale::seeds::PRESALE_PREFIX.as_ref(),
            base.as_ref(),
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
    base: &Pubkey,
    program_id: &Pubkey,
) -> Pubkey {
    let presale = derive_presale(mint, quote, base, program_id);
    Pubkey::find_program_address(
        &[
            presale::seeds::FIXED_PRICE_PRESALE_PARAM_PREFIX.as_ref(),
            presale.as_ref(),
        ],
        program_id,
    )
    .0
}

pub fn derive_escrow(presale: &Pubkey, owner: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            presale::seeds::ESCROW_PREFIX.as_ref(),
            presale.as_ref(),
            owner.as_ref(),
        ],
        program_id,
    )
    .0
}

pub fn derive_operator(creator: &Pubkey, operator: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            presale::seeds::OPERATOR_PREFIX.as_ref(),
            creator.as_ref(),
            operator.as_ref(),
        ],
        program_id,
    )
    .0
}

pub fn derive_merkle_root_config(presale: &Pubkey, version: u64, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            presale::seeds::MERKLE_ROOT_CONFIG_PREFIX.as_ref(),
            presale.as_ref(),
            version.to_le_bytes().as_ref(),
        ],
        program_id,
    )
    .0
}

pub fn derive_merkle_proof_metadata(presale: &Pubkey, program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            presale::seeds::MERKLE_PROOF_METADATA_PREFIX.as_ref(),
            presale.as_ref(),
        ],
        program_id,
    )
    .0
}
