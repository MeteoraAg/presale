macro_rules! presale_authority_seeds {
    () => {
        &[
            crate::constants::seeds::PRESALE_AUTHORITY_PREFIX.as_ref(),
            &[crate::const_pda::presale_authority::BUMP],
        ]
    };
}
