use std::rc::Rc;

use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;

use crate::helpers::{
    derive_event_authority, derive_permissioned_server_metadata, process_transaction,
};

#[derive(Clone)]
pub struct ClosePermissionedServerMetadataArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn handle_close_permissioned_server_metadata_ix(
    args: ClosePermissionedServerMetadataArgs,
) -> Instruction {
    let ClosePermissionedServerMetadataArgs { presale, owner } = args;

    let ix_data = presale::instruction::ClosePermissionedServerMetadata {}.data();

    let permissioned_server_metadata = derive_permissioned_server_metadata(&presale, &presale::ID);

    let accounts = presale::accounts::ClosePermissionedServerMetadataCtx {
        presale,
        permissioned_server_metadata,
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

pub fn handle_close_permissioned_server_metadata(
    lite_svm: &mut LiteSVM,
    args: ClosePermissionedServerMetadataArgs,
) {
    let instruction = handle_close_permissioned_server_metadata_ix(args.clone());

    let ClosePermissionedServerMetadataArgs { owner, .. } = args;
    let owner_pubkey = owner.pubkey();

    process_transaction(lite_svm, &[instruction], Some(&owner_pubkey), &[&owner]).unwrap();
}
