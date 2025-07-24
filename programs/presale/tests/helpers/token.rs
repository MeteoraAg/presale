use crate::helpers::process_transaction;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::{
    instruction::Instruction, program_pack::Pack, signature::Keypair, signer::Signer,
    system_instruction::create_account,
};
use anchor_lang::prelude::Rent;
use anchor_spl::token::spl_token::instruction::initialize_mint;
use litesvm::LiteSVM;
use std::rc::Rc;

#[derive(Clone)]
pub struct CreateTokenArgs {
    pub mint: Rc<Keypair>,
    pub mint_authority: Rc<Keypair>,
    pub payer: Rc<Keypair>,
    pub decimals: u8,
}

pub fn create_token_ix(lite_svm: &mut LiteSVM, args: CreateTokenArgs) -> Vec<Instruction> {
    let CreateTokenArgs {
        mint,
        mint_authority,
        payer,
        decimals,
    } = args;

    let mint_pubkey = mint.pubkey();
    let mint_authority_pubkey = mint_authority.pubkey();
    let payer_pubkey = payer.pubkey();

    let rent = lite_svm.get_sysvar::<Rent>();

    let space = anchor_spl::token::spl_token::state::Mint::LEN;
    let lamports = rent.minimum_balance(space);

    let create_account_ix = create_account(
        &payer_pubkey,
        &mint_pubkey,
        lamports,
        space as u64,
        &anchor_spl::token::spl_token::ID,
    );

    let initialize_mint_ix = initialize_mint(
        &anchor_spl::token::spl_token::ID,
        &mint_pubkey,
        &mint_authority_pubkey,
        None,
        decimals,
    )
    .expect("Failed to create initialize_mint instruction");

    vec![create_account_ix, initialize_mint_ix]
}

pub fn create_token(lite_svm: &mut LiteSVM, args: CreateTokenArgs) {
    let instructions = create_token_ix(lite_svm, args.clone());
    let CreateTokenArgs { mint, payer, .. } = args;
    process_transaction(
        lite_svm,
        &instructions,
        Some(&payer.pubkey()),
        &[&payer, &mint],
    )
    .unwrap();
}

pub fn get_program_id_from_token_flag(token_flag: u8) -> Pubkey {
    match token_flag {
        0 => anchor_spl::token::ID,
        1 => anchor_spl::token_2022::ID,
        _ => panic!("Unsupported token flag"),
    }
}
