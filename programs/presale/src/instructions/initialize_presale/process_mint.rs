use crate::instructions::TokenomicArgs;
use anchor_lang::prelude::*;
use anchor_spl::{token_2022::spl_token_2022::instruction::AuthorityType, token_interface::*};

pub struct ProcessMintTokenSupplyArgs<'a, 'info> {
    pub mint: AccountInfo<'info>,
    pub base_vault: AccountInfo<'info>,
    pub presale_authority: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub tokenomic: &'a TokenomicArgs,
}

pub fn process_mint_token_supply(params: ProcessMintTokenSupplyArgs) -> Result<()> {
    let ProcessMintTokenSupplyArgs {
        mint,
        base_vault,
        presale_authority,
        token_program,
        tokenomic,
    } = params;

    let signer_seeds = &[&presale_authority_seeds!()[..]];

    // 1. Mint
    anchor_spl::token_interface::mint_to(
        CpiContext::new_with_signer(
            token_program.clone(),
            MintTo {
                mint: mint.clone(),
                to: base_vault.clone(),
                authority: presale_authority.clone(),
            },
            signer_seeds,
        ),
        tokenomic.get_total_supply()?,
    )?;

    // 2. Give up mint authority
    anchor_spl::token_interface::set_authority(
        CpiContext::new_with_signer(
            token_program,
            SetAuthority {
                current_authority: presale_authority,
                account_or_mint: mint,
            },
            signer_seeds,
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    Ok(())
}
