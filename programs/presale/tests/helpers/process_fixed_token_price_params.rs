use std::rc::Rc;

use crate::helpers::*;
use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use presale::UnsoldTokenAction;

#[derive(Clone)]
pub struct HandleInitializeFixedTokenPricePresaleParamsArgs {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub q_price: u128,
    pub unsold_token_action: UnsoldTokenAction,
    pub owner: Pubkey,
    pub payer: Rc<Keypair>,
    pub base: Pubkey,
}

pub fn create_initialize_fixed_token_price_presale_params_args_ix(
    args: HandleInitializeFixedTokenPricePresaleParamsArgs,
) -> Instruction {
    let HandleInitializeFixedTokenPricePresaleParamsArgs {
        base_mint,
        quote_mint,
        q_price,
        unsold_token_action,
        owner,
        payer,
        base,
    } = args;

    let presale = derive_presale(&base_mint, &quote_mint, &base, &presale::ID);
    let event_authority = derive_event_authority(&presale::ID);
    let fixed_price_presale_params =
        derive_fixed_price_presale_args(&base_mint, &quote_mint, &base, &presale::ID);

    let ix_data = presale::instruction::InitializeFixedPricePresaleArgs {
        params: presale::InitializeFixedPricePresaleExtraArgs {
            q_price,
            presale,
            unsold_token_action: unsold_token_action.into(),
            ..Default::default()
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

    instruction
}

pub fn handle_initialize_fixed_token_price_presale_params(
    lite_svm: &mut LiteSVM,
    args: HandleInitializeFixedTokenPricePresaleParamsArgs,
) {
    let instruction = create_initialize_fixed_token_price_presale_params_args_ix(args.clone());
    let HandleInitializeFixedTokenPricePresaleParamsArgs { payer, .. } = args;
    process_transaction(lite_svm, &[instruction], Some(&payer.pubkey()), &[&payer]).unwrap();
}

#[derive(Clone)]
pub struct HandleCloseFixedTokenPricePresaleParamsArgs {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub owner: Rc<Keypair>,
    pub base: Pubkey,
}

pub fn create_close_fixed_token_price_presale_params_ix(
    args: HandleCloseFixedTokenPricePresaleParamsArgs,
) -> Instruction {
    let HandleCloseFixedTokenPricePresaleParamsArgs {
        base_mint,
        quote_mint,
        owner,
        base,
    } = args;

    let event_authority = derive_event_authority(&presale::ID);
    let fixed_price_presale_args =
        derive_fixed_price_presale_args(&base_mint, &quote_mint, &base, &presale::ID);

    let ix_data = presale::instruction::CloseFixedPricePresaleArgs {}.data();

    let accounts = presale::accounts::CloseFixedPricePresaleArgsCtx {
        fixed_price_presale_args,
        owner: owner.pubkey(),
        event_authority,
        program: presale::ID,
    };

    Instruction {
        program_id: presale::ID,
        accounts: accounts.to_account_metas(None),
        data: ix_data,
    }
}

pub fn handle_close_fixed_token_price_presale_params_err(
    lite_svm: &mut LiteSVM,
    args: HandleCloseFixedTokenPricePresaleParamsArgs,
) -> FailedTransactionMetadata {
    let instruction = create_close_fixed_token_price_presale_params_ix(args.clone());
    let HandleCloseFixedTokenPricePresaleParamsArgs { owner, .. } = args;
    process_transaction(lite_svm, &[instruction], Some(&owner.pubkey()), &[&owner]).unwrap_err()
}

pub fn handle_close_fixed_token_price_presale_params(
    lite_svm: &mut LiteSVM,
    args: HandleCloseFixedTokenPricePresaleParamsArgs,
) {
    let instruction = create_close_fixed_token_price_presale_params_ix(args.clone());
    let HandleCloseFixedTokenPricePresaleParamsArgs { owner, .. } = args;
    process_transaction(lite_svm, &[instruction], Some(&owner.pubkey()), &[&owner]).unwrap();
}
