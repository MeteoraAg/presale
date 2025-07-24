use std::rc::Rc;

use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Keypair};
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;

use crate::helpers::{derive_event_authority, derive_operator, process_transaction};

#[derive(Clone)]
pub struct HandleCreateOperatorArgs {
    pub owner: Rc<Keypair>,
    pub operator: Pubkey,
}

pub fn create_operator_ix(args: HandleCreateOperatorArgs) -> Instruction {
    let HandleCreateOperatorArgs { owner, operator } = args;

    let owner_pubkey = owner.pubkey();

    let ix_data = presale::instruction::CreateOperator {}.data();

    let operator_pda = derive_operator(&owner_pubkey, &operator, &presale::ID);

    let accounts = presale::accounts::CreateOperatorCtx {
        operator: operator_pda,
        operator_owner: operator,
        creator: owner_pubkey,
        system_program: anchor_lang::solana_program::system_program::ID,
        program: presale::ID,
        event_authority: derive_event_authority(&presale::ID),
    }
    .to_account_metas(None);

    Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    }
}

pub fn handle_create_operator(lite_svm: &mut LiteSVM, args: HandleCreateOperatorArgs) {
    let instruction = create_operator_ix(args.clone());

    let HandleCreateOperatorArgs { owner, .. } = args;
    process_transaction(lite_svm, &[instruction], Some(&owner.pubkey()), &[&owner]).unwrap();
}
