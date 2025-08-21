use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use std::rc::Rc;

use crate::helpers::{derive_escrow, derive_event_authority, process_transaction};

#[derive(Clone)]
pub struct HandleCloseEscrowArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
    pub registry_index: u8,
}

pub fn handle_close_escrow_ix(args: HandleCloseEscrowArgs) -> Vec<Instruction> {
    let HandleCloseEscrowArgs {
        owner,
        presale,
        registry_index,
    } = args;

    let owner_pubkey = owner.pubkey();
    let escrow = derive_escrow(&presale, &owner_pubkey, registry_index, &presale::ID);

    let ix_data = presale::instruction::CloseEscrow {}.data();

    let accounts = presale::accounts::CloseEscrowCtx {
        presale,
        escrow,
        owner: owner_pubkey,
        event_authority: derive_event_authority(&presale::ID),
        program: presale::ID,
        rent_receiver: owner_pubkey, // The rent receiver is the owner in this case.
    }
    .to_account_metas(None);

    let ix = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    vec![ix]
}

pub fn handle_close_escrow(lite_svm: &mut LiteSVM, args: HandleCloseEscrowArgs) {
    let instructions = handle_close_escrow_ix(args.clone());
    let HandleCloseEscrowArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();
    process_transaction(lite_svm, &instructions, Some(&owner_pubkey), &[&owner]).unwrap();
}

pub fn handle_close_escrow_err(
    lite_svm: &mut LiteSVM,
    args: HandleCloseEscrowArgs,
) -> FailedTransactionMetadata {
    let instructions = handle_close_escrow_ix(args.clone());
    let HandleCloseEscrowArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();
    process_transaction(lite_svm, &instructions, Some(&owner_pubkey), &[&owner]).unwrap_err()
}
