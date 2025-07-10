use crate::*;

mod params;
pub use params::*;

mod process_initialize_presale;
pub use process_initialize_presale::*;

mod process_initialize_presale_token2022;
pub use process_initialize_presale_token2022::*;

mod process_create_metaplex_metadata;

mod process_mint;

mod process_create_presale_vault;

fn ensure_whitelisted_quote(mint: Pubkey) -> Result<()> {
    require!(QUOTE_MINTS.contains(&mint), PresaleError::InvalidQuoteMint);
    Ok(())
}
