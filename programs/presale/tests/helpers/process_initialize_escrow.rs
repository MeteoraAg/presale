use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use std::rc::Rc;

use crate::helpers::{derive_escrow, derive_event_authority, derive_operator, process_transaction};

#[derive(Clone)]
pub struct HandleCreatePermissionedEscrowWithMerkleProofArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
    pub merkle_root_config: Pubkey,
    pub proof: Vec<[u8; 32]>,
}

pub fn create_permissioned_escrow_with_merkle_proof_ix(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionedEscrowWithMerkleProofArgs,
) -> Option<Instruction> {
    let HandleCreatePermissionedEscrowWithMerkleProofArgs {
        presale,
        owner,
        proof,
        merkle_root_config,
    } = args;

    let owner_pubkey = owner.pubkey();
    let escrow = derive_escrow(&presale, &owner_pubkey, &presale::ID);

    let escrow_account = lite_svm.get_account(&escrow);

    if escrow_account.is_some() {
        return None; // Escrow account already exists
    }

    let ix_data = presale::instruction::CreatePermissionedEscrowWithMerkleProof { proof }.data();

    let accounts = presale::accounts::CreatePermissionedEscrowWithMerkleProofCtx {
        escrow,
        merkle_root_config,
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

pub fn handle_create_permissioned_escrow_with_merkle_proof(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionedEscrowWithMerkleProofArgs,
) {
    let instruction =
        create_permissioned_escrow_with_merkle_proof_ix(lite_svm, args.clone()).unwrap();
    let HandleCreatePermissionedEscrowWithMerkleProofArgs { owner, .. } = args;
    process_transaction(lite_svm, &[instruction], Some(&owner.pubkey()), &[&owner]).unwrap();
}

pub fn handle_create_permissioned_escrow_with_merkle_proof_err(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionedEscrowWithMerkleProofArgs,
) -> FailedTransactionMetadata {
    let instruction =
        create_permissioned_escrow_with_merkle_proof_ix(lite_svm, args.clone()).unwrap();
    let HandleCreatePermissionedEscrowWithMerkleProofArgs { owner, .. } = args;
    process_transaction(lite_svm, &[instruction], Some(&owner.pubkey()), &[&owner]).unwrap_err()
}

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
    let escrow = derive_escrow(&presale, &owner_pubkey, &presale::ID);

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

#[derive(Clone)]
pub struct HandleCreatePermissionedEscrowWithOperatorArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
    pub vault_owner: Pubkey,
    pub operator: Rc<Keypair>,
}

pub fn create_permissioned_escrow_with_operator_ix(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionedEscrowWithOperatorArgs,
) -> Option<Instruction> {
    let HandleCreatePermissionedEscrowWithOperatorArgs {
        presale,
        owner,
        vault_owner,
        operator,
    } = args;

    let owner_pubkey = owner.pubkey();
    let escrow = derive_escrow(&presale, &owner_pubkey, &presale::ID);

    let escrow_account = lite_svm.get_account(&escrow);

    if escrow_account.is_some() {
        return None; // Escrow account already exists
    }

    let operator_pda = derive_operator(&vault_owner, &operator.pubkey(), &presale::ID);
    let ix_data = presale::instruction::CreatePermissionedEscrowWithCreator {}.data();

    let accounts = presale::accounts::CreatePermissionedEscrowWithCreatorCtx {
        escrow,
        owner: owner_pubkey,
        system_program: anchor_lang::solana_program::system_program::ID,
        program: presale::ID,
        presale,
        payer: owner_pubkey,
        operator: operator_pda,
        operator_owner: operator.pubkey(),
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

pub fn handle_create_permissioned_escrow_with_operator(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionedEscrowWithOperatorArgs,
) {
    let instruction = create_permissioned_escrow_with_operator_ix(lite_svm, args.clone());
    if instruction.is_none() {
        return; // Escrow account already exists
    }

    let HandleCreatePermissionedEscrowWithOperatorArgs {
        owner, operator, ..
    } = args;
    process_transaction(
        lite_svm,
        &[instruction.unwrap()],
        Some(&owner.pubkey()),
        &[&owner, &operator],
    )
    .unwrap();
}

pub fn handle_create_permissioned_escrow_with_operator_err(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionedEscrowWithOperatorArgs,
) -> FailedTransactionMetadata {
    let instruction = create_permissioned_escrow_with_operator_ix(lite_svm, args.clone()).unwrap();

    let HandleCreatePermissionedEscrowWithOperatorArgs {
        owner, operator, ..
    } = args;
    process_transaction(
        lite_svm,
        &[instruction],
        Some(&owner.pubkey()),
        &[&owner, &operator],
    )
    .unwrap_err()
}

pub fn handle_create_permissionless_escrow(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionlessEscrowArgs,
) {
    let instruction = create_permissionless_escrow_ix(lite_svm, args.clone()).unwrap();
    let HandleCreatePermissionlessEscrowArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();
    process_transaction(lite_svm, &[instruction], Some(&owner_pubkey), &[&owner]).unwrap();
}

pub fn handle_create_permissionless_escrow_err(
    lite_svm: &mut LiteSVM,
    args: HandleCreatePermissionlessEscrowArgs,
) -> FailedTransactionMetadata {
    let instruction = create_permissionless_escrow_ix(lite_svm, args.clone()).unwrap();
    let HandleCreatePermissionlessEscrowArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();
    process_transaction(lite_svm, &[instruction], Some(&owner_pubkey), &[&owner]).unwrap_err()
}
