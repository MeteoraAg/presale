use std::rc::Rc;

use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::{types::FailedTransactionMetadata, LiteSVM};

use crate::helpers::{derive_event_authority, derive_merkle_proof_metadata, process_transaction};

#[derive(Clone)]
pub struct CreateMerkleProofMetadataArgs {
    pub proof_url: String,
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn handle_create_merkle_proof_metadata_ix(args: CreateMerkleProofMetadataArgs) -> Instruction {
    let CreateMerkleProofMetadataArgs {
        proof_url,
        presale,
        owner,
    } = args;

    let ix_data = presale::instruction::CreateMerkleProofMetadata {
        proof_url: proof_url.clone(),
    }
    .data();

    let merkle_proof_metadata = derive_merkle_proof_metadata(&presale, &presale::ID);

    let accounts = presale::accounts::CreateMerkleProofMetadataCtx {
        presale,
        merkle_proof_metadata,
        owner: owner.pubkey(),
        system_program: anchor_client::solana_sdk::system_program::ID,
        event_authority: derive_event_authority(&presale::ID),
        program: presale::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    instruction
}

pub fn handle_create_merkle_proof_metadata(
    lite_svm: &mut LiteSVM,
    args: CreateMerkleProofMetadataArgs,
) {
    let instruction = handle_create_merkle_proof_metadata_ix(args.clone());

    let CreateMerkleProofMetadataArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();

    process_transaction(lite_svm, &[instruction], Some(&owner_pubkey), &[&owner]).unwrap();
}

pub fn handle_create_merkle_proof_metadata_err(
    lite_svm: &mut LiteSVM,
    args: CreateMerkleProofMetadataArgs,
) -> FailedTransactionMetadata {
    let instruction = handle_create_merkle_proof_metadata_ix(args.clone());

    let CreateMerkleProofMetadataArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();

    process_transaction(lite_svm, &[instruction], Some(&owner_pubkey), &[&owner]).unwrap_err()
}
