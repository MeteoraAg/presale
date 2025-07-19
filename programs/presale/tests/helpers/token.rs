use crate::helpers::process_transaction;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::{
    program_pack::Pack, signature::Keypair, signer::Signer, system_instruction::create_account,
};
use anchor_lang::prelude::Rent;
use anchor_spl::token::spl_token::instruction::initialize_mint;
use litesvm::LiteSVM;
use std::rc::Rc;

pub struct CreateTokenArgs<'a> {
    pub lite_svm: &'a mut LiteSVM,
    pub mint: Rc<Keypair>,
    pub mint_authority: Rc<Keypair>,
    pub payer: Rc<Keypair>,
    pub decimals: u8,
}

pub fn create_token(args: CreateTokenArgs) {
    let CreateTokenArgs {
        lite_svm,
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

    process_transaction(
        lite_svm,
        &[create_account_ix, initialize_mint_ix],
        Some(&payer_pubkey),
        &[&payer, &mint],
    )
    .unwrap();
}

pub fn get_program_id_from_token_flag(token_flag: u8) -> Pubkey {
    match token_flag {
        0 => anchor_spl::token_2022::ID,
        1 => anchor_spl::token_interface::ID,
        _ => panic!("Unsupported token flag"),
    }
}
