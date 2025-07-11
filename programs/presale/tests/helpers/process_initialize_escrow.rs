use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use litesvm::LiteSVM;
use std::rc::Rc;

use crate::helpers::{derive_escrow, derive_event_authority, process_transaction};

#[derive(Clone)]
pub struct HandleCreatePermissionlessEscrowArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn create_permissionless_escrow_ix(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionlessEscrowArgs,
) -> Option<Instruction> {
    let HandleCreatePermissionlessEscrowArgs { presale, owner } = args;

    let owner_pubkey = owner.pubkey();
    let escrow = derive_escrow(presale, owner_pubkey, &presale::ID);

    let escrow_account = lite_svm.get_account(&escrow);

    if escrow_account.is_some() {
        return None; // Escrow account already exists
    }

    let ix_data = presale::instruction::CreatePermissionlessEscrow {}.data();

    let accounts = presale::accounts::CreatePermissionlessEscrowCtx {
        escrow,
        owner: owner_pubkey,
        system_program: anchor_lang::solana_program::system_program::ID,
        program: presale::ID,
        presale,
        payer: owner_pubkey,
        event_authority: derive_event_authority(&presale::ID),
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    Some(instruction)
}

pub fn handle_create_permissionless_escrow(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionlessEscrowArgs,
) {
    let instruction = create_permissionless_escrow_ix(lite_svm, args.clone()).unwrap();
    let HandleCreatePermissionlessEscrowArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();
    process_transaction(lite_svm, &[instruction], Some(&owner_pubkey), &[&owner]);
}
