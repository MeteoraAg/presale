use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use litesvm::LiteSVM;
use std::rc::Rc;

use crate::helpers::{derive_escrow, derive_event_authority, process_transaction};

pub struct HandleCloseEscrowArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn handle_close_escrow(lite_svm: &mut LiteSVM, args: HandleCloseEscrowArgs) {
    let HandleCloseEscrowArgs { owner, presale } = args;

    let owner_pubkey = owner.pubkey();
    let escrow = derive_escrow(&presale, &owner_pubkey, &presale::ID);

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

    process_transaction(lite_svm, &[ix], Some(&owner_pubkey), &[&owner]).unwrap();
}
