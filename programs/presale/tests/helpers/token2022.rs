use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::{program_pack::Pack, system_instruction::create_account};
use anchor_lang::prelude::Rent;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use anchor_spl::token_2022::spl_token_2022::instruction::{initialize_mint, mint_to};
use litesvm::LiteSVM;

use crate::*;

pub struct CreateToken2022Args<'a> {
    pub lite_svm: &'a mut LiteSVM,
    pub mint: Rc<Keypair>,
    pub mint_authority: Rc<Keypair>,
    pub payer: Rc<Keypair>,
    pub decimals: u8,
}

// TODO: Support extension
pub fn create_token_2022(args: CreateToken2022Args) {
    let CreateToken2022Args {
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

    let space = anchor_spl::token_2022::spl_token_2022::state::Mint::LEN;
    let lamports = rent.minimum_balance(space);

    let create_account_ix = create_account(
        &payer_pubkey,
        &mint_pubkey,
        lamports,
        space as u64,
        &anchor_spl::token_2022::spl_token_2022::ID,
    );

    let initialize_mint_ix = initialize_mint(
        &anchor_spl::token_2022::spl_token_2022::ID,
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
    );
}

pub struct MintToken2022ToArgs<'a> {
    pub lite_svm: &'a mut LiteSVM,
    pub mint: Pubkey,
    pub amount: u64,
    pub destination: Pubkey,
    pub mint_authority: Rc<Keypair>,
}

fn mint_token2022_to(args: MintToken2022ToArgs) {
    let MintToken2022ToArgs {
        lite_svm,
        mint,
        amount,
        destination,
        mint_authority,
    } = args;

    let mint_authority_pubkey = mint_authority.pubkey();

    let destination_ata = get_associated_token_address_with_program_id(
        &destination,
        &mint,
        &anchor_spl::token_2022::spl_token_2022::ID,
    );

    let create_ata_ix = create_associated_token_account_idempotent(
        &mint_authority_pubkey,
        &destination,
        &mint,
        &anchor_spl::token_2022::spl_token_2022::ID,
    );

    let mint_ix = mint_to(
        &anchor_spl::token_2022::spl_token_2022::ID,
        &mint,
        &destination_ata,
        &mint_authority_pubkey,
        &[&mint_authority_pubkey],
        amount,
    )
    .expect("Failed to create mint_to instruction");

    process_transaction(
        lite_svm,
        &[create_ata_ix, mint_ix],
        Some(&mint_authority_pubkey),
        &[&mint_authority],
    );
}
