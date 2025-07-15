use std::rc::Rc;

use crate::helpers::*;
use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use litesvm::LiteSVM;
use presale::UnsoldTokenAction;

#[derive(Clone)]
pub struct HandleInitializeFixedTokenPricePresaleParamsArgs {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub q_price: u128,
    pub unsold_token_action: UnsoldTokenAction,
    pub owner: Pubkey,
    pub payer: Rc<Keypair>,
}

pub fn handle_initialize_fixed_token_price_presale_params(
    lite_svm: &mut LiteSVM,
    args: HandleInitializeFixedTokenPricePresaleParamsArgs,
) {
    let HandleInitializeFixedTokenPricePresaleParamsArgs {
        base_mint,
        quote_mint,
        q_price,
        unsold_token_action,
        owner,
        payer,
    } = args;

    let presale = derive_presale(&base_mint, &quote_mint, &presale::ID);
    let event_authority = derive_event_authority(&presale::ID);
    let fixed_price_presale_params =
        derive_fixed_price_presale_args(&base_mint, &quote_mint, &presale::ID);

    let ix_data = presale::instruction::InitializeFixedPricePresaleArgs {
        params: presale::InitializeFixedPricePresaleExtraArgs {
            q_price,
            presale,
            unsold_token_action: unsold_token_action.into(),
        },
    }
    .data();

    let accounts = presale::accounts::InitializeFixedPricePresaleArgsCtx {
        fixed_price_presale_params,
        owner,
        payer: payer.pubkey(),
        system_program: anchor_lang::solana_program::system_program::ID,
        event_authority,
        program: presale::ID,
    };

    let instruction = Instruction {
        program_id: presale::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data,
    };

    process_transaction(lite_svm, &[instruction], Some(&payer.pubkey()), &[&payer]);
}

pub struct HandleCloseFixedTokenPricePresaleParamsArgs {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn handle_close_fixed_token_price_presale_params(
    lite_svm: &mut LiteSVM,
    args: HandleCloseFixedTokenPricePresaleParamsArgs,
) {
    let HandleCloseFixedTokenPricePresaleParamsArgs {
        base_mint,
        quote_mint,
        owner,
    } = args;

    let event_authority = derive_event_authority(&presale::ID);
    let fixed_price_presale_args =
        derive_fixed_price_presale_args(&base_mint, &quote_mint, &presale::ID);

    let ix_data = presale::instruction::CloseFixedPricePresaleArgs {}.data();

    let accounts = presale::accounts::CloseFixedPricePresaleArgsCtx {
        fixed_price_presale_args,
        owner: owner.pubkey(),
        event_authority,
        program: presale::ID,
    };

    let instruction = Instruction {
        program_id: presale::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data,
    };

    process_transaction(lite_svm, &[instruction], Some(&owner.pubkey()), &[&owner]);
}
