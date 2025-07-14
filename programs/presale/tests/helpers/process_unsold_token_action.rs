use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use anchor_spl::associated_token::{
    get_associated_token_address_with_program_id,
    spl_associated_token_account::instruction::create_associated_token_account_idempotent,
};
use litesvm::LiteSVM;
use presale::Presale;
use std::rc::Rc;

use crate::helpers::{derive_event_authority, process_transaction, LiteSVMExt};

pub struct HandlePerformUnsoldTokenActionArgs {
    pub presale: Pubkey,
    pub creator: Rc<Keypair>,
}

pub fn handle_perform_unsold_token_action(
    lite_svm: &mut LiteSVM,
    args: HandlePerformUnsoldTokenActionArgs,
) {
    let HandlePerformUnsoldTokenActionArgs { creator, presale } = args;

    let creator_pubkey = creator.pubkey();

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale)
        .unwrap();

    let token_program = lite_svm
        .get_account(&presale_state.base_mint)
        .unwrap()
        .owner;

    let ix_data = presale::instruction::PerformUnsoldBaseTokenAction {}.data();

    let creator_base_token = get_associated_token_address_with_program_id(
        &creator_pubkey,
        &presale_state.base_mint,
        &token_program,
    );

    let create_creator_base_token_ix = create_associated_token_account_idempotent(
        &creator_pubkey,
        &creator_pubkey,
        &presale_state.base_mint,
        &token_program,
    );

    let accounts = presale::accounts::PerformUnsoldBaseTokenActionCtx {
        presale,
        creator_base_token,
        event_authority: derive_event_authority(&presale::ID),
        token_program,
        program: presale::ID,
        base_mint: presale_state.base_mint,
        base_token_vault: presale_state.base_token_vault,
        presale_authority: presale::presale_authority::ID,
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    process_transaction(
        lite_svm,
        &[create_creator_base_token_ix, ix],
        Some(&creator_pubkey),
        &[&creator],
    );
}
