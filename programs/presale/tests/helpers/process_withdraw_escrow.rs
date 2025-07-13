use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use litesvm::LiteSVM;
use presale::Presale;
use std::rc::Rc;

use crate::helpers::{derive_escrow, derive_event_authority, process_transaction, LiteSVMExt};

pub struct HandleEscrowWithdrawArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
    pub amount: u64,
}

pub fn handle_escrow_withdraw(lite_svm: &mut LiteSVM, args: HandleEscrowWithdrawArgs) {
    let HandleEscrowWithdrawArgs {
        owner,
        presale,
        amount,
    } = args;
    let owner_pubkey = owner.pubkey();

    let mut instructions = vec![];

    let presale_state: Presale = lite_svm.get_deserialized_account(&presale).unwrap();

    let quote_token_program = lite_svm
        .get_account(&presale_state.quote_mint)
        .unwrap()
        .owner;

    let escrow = derive_escrow(presale, owner_pubkey, &presale::ID);
    let owner_quote_token = get_associated_token_address_with_program_id(
        &owner_pubkey,
        &presale_state.quote_mint,
        &quote_token_program,
    );

    let ix_data = presale::instruction::Withdraw { amount }.data();
    let accounts = presale::accounts::WithdrawCtx {
        quote_mint: presale_state.quote_mint,
        quote_token_vault: presale_state.quote_token_vault,
        owner_quote_token,
        owner: owner.pubkey(),
        escrow,
        token_program: quote_token_program,
        program: presale::ID,
        presale,
        event_authority: derive_event_authority(&presale::ID),
        presale_authority: presale::presale_authority::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };
    instructions.push(instruction);

    process_transaction(lite_svm, &instructions, Some(&owner_pubkey), &[&owner]);
}
