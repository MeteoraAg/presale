use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use litesvm::LiteSVM;
use std::rc::Rc;

use crate::helpers::{derive_escrow, derive_event_authority, process_transaction};

#[derive(Clone)]
pub struct HandleEscrowRefreshArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn create_refresh_escrow_ix(args: HandleEscrowRefreshArgs) -> Vec<Instruction> {
    let HandleEscrowRefreshArgs { owner, presale } = args;
    let owner_pubkey = owner.pubkey();

    let mut instructions = vec![];

    let escrow = derive_escrow(&presale, &owner_pubkey, &presale::ID);

    let ix_data = presale::instruction::RefreshEscrow {}.data();
    let accounts = presale::accounts::RefreshEscrowCtx {
        escrow,
        program: presale::ID,
        presale,
        event_authority: derive_event_authority(&presale::ID),
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };
    instructions.push(instruction);

    instructions
}

pub fn handle_escrow_refresh(lite_svm: &mut LiteSVM, args: HandleEscrowRefreshArgs) {
    let instructions = create_refresh_escrow_ix(args.clone());
    let HandleEscrowRefreshArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();
    process_transaction(lite_svm, &instructions, Some(&owner_pubkey), &[&owner]).unwrap();
}
