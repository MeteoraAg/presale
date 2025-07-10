use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use litesvm::LiteSVM;
use std::rc::Rc;

use crate::helpers::{derive_escrow, derive_event_authority, process_transaction};

pub struct HandleCreatePermissionlessEscrowArgs<'a> {
    pub lite_svm: &'a mut LiteSVM,
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn handle_create_permissionless_escrow(args: HandleCreatePermissionlessEscrowArgs) {
    let HandleCreatePermissionlessEscrowArgs {
        lite_svm,
        presale,
        owner,
    } = args;

    let owner_pubkey = owner.pubkey();
    let escrow = derive_escrow(presale, owner_pubkey, &presale::ID);

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

    process_transaction(lite_svm, &[instruction], Some(&owner_pubkey), &[&owner]);
}
