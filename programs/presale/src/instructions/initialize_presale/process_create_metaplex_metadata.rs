use anchor_lang::prelude::*;
use mpl_token_metadata::types::DataV2;

pub struct ProcessCreateTokenMetadataArgs<'a, 'info> {
    pub system_program: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,             // signer
    pub presale_authority: AccountInfo<'info>, // signer
    pub mint: AccountInfo<'info>,              // signer
    pub metadata_program: AccountInfo<'info>,
    pub mint_metadata: AccountInfo<'info>,
    pub name: &'a str,
    pub symbol: &'a str,
    pub uri: &'a str,
}

pub fn process_create_mpl_token_metadata(params: ProcessCreateTokenMetadataArgs) -> Result<()> {
    let presale_authority_seeds = presale_authority_seeds!();
    let mut builder = mpl_token_metadata::instructions::CreateMetadataAccountV3CpiBuilder::new(
        &params.metadata_program,
    );
    builder.mint(&params.mint);
    builder.metadata(&params.mint_metadata);
    builder.is_mutable(false);
    builder.mint_authority(&params.presale_authority);
    builder.update_authority(&params.presale_authority, true); // TODO transfer to creator when presale done
    builder.payer(&params.payer);
    builder.system_program(&params.system_program);
    let data = DataV2 {
        collection: None,
        creators: None,
        name: params.name.to_string(),
        symbol: params.symbol.to_string(),
        seller_fee_basis_points: 0,
        uses: None,
        uri: params.uri.to_string(),
    };
    builder.data(data);
    builder.invoke_signed(&[&presale_authority_seeds[..]])?;

    Ok(())
}
