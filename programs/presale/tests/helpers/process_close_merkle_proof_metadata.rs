use std::rc::Rc;

use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;

use crate::helpers::{derive_event_authority, derive_merkle_proof_metadata, process_transaction};

#[derive(Clone)]
pub struct CloseMerkleProofMetadataArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn handle_close_merkle_proof_metadata_ix(args: CloseMerkleProofMetadataArgs) -> Instruction {
    let CloseMerkleProofMetadataArgs { presale, owner } = args;

    let ix_data = presale::instruction::CloseMerkleProofMetadata {}.data();

    let merkle_proof_metadata = derive_merkle_proof_metadata(&presale, &presale::ID);

    let accounts = presale::accounts::CloseMerkleProofMetadataCtx {
        presale,
        merkle_proof_metadata,
        owner: owner.pubkey(),
        event_authority: derive_event_authority(&presale::ID),
        program: presale::ID,
        rent_receiver: owner.pubkey(), // Rent receiver is the owner in this case
    }
    .to_account_metas(None);

    Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    }
}

pub fn handle_close_merkle_proof_metadata(
    lite_svm: &mut LiteSVM,
    args: CloseMerkleProofMetadataArgs,
) {
    let instruction = handle_close_merkle_proof_metadata_ix(args.clone());

    let CloseMerkleProofMetadataArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();

    process_transaction(lite_svm, &[instruction], Some(&owner_pubkey), &[&owner]).unwrap();
}
