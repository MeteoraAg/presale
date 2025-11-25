use std::rc::Rc;

use crate::helpers::{derive_event_authority, derive_merkle_root_config, process_transaction};
use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::{types::FailedTransactionMetadata, LiteSVM};

#[derive(Clone)]
pub struct HandleCloseMerkleRootConfigArgs {
    pub presale: Pubkey,
    pub version: u8,
    pub creator: Rc<Keypair>,
}

pub struct HandleCloseMerkleRootConfigWrapper {
    pub instructions: presale::instruction::CloseMerkleRootConfig,
    pub accounts: presale::accounts::CloseMerkleRootConfigCtx,
}

pub fn handle_close_merkle_root_config_wrapper(
    args: HandleCloseMerkleRootConfigArgs,
) -> HandleCloseMerkleRootConfigWrapper {
    let HandleCloseMerkleRootConfigArgs {
        presale,
        version,
        creator,
    } = args;

    let creator_pubkey = creator.pubkey();

    let merkle_root_config = derive_merkle_root_config(&presale, version.into(), &presale::ID);

    let accounts = presale::accounts::CloseMerkleRootConfigCtx {
        presale,
        merkle_root_config,
        rent_receiver: creator_pubkey,
        event_authority: derive_event_authority(&presale::ID),
        creator: creator_pubkey,
        program: presale::ID,
    };

    HandleCloseMerkleRootConfigWrapper {
        instructions: presale::instruction::CloseMerkleRootConfig {},
        accounts,
    }
}

pub fn handle_close_merkle_root_config_ix(
    args: HandleCloseMerkleRootConfigArgs,
) -> Vec<Instruction> {
    let HandleCloseMerkleRootConfigWrapper {
        instructions,
        accounts,
    } = handle_close_merkle_root_config_wrapper(args);

    let instruction = Instruction {
        program_id: presale::ID,
        accounts: accounts.to_account_metas(None),
        data: instructions.data(),
    };

    vec![instruction]
}

pub fn handle_close_merkle_root_config(
    lite_svm: &mut LiteSVM,
    args: HandleCloseMerkleRootConfigArgs,
) {
    let instructions = handle_close_merkle_root_config_ix(args.clone());
    let HandleCloseMerkleRootConfigArgs { creator, .. } = args;
    let creator_pubkey = creator.pubkey();
    process_transaction(lite_svm, &instructions, Some(&creator_pubkey), &[&creator]).unwrap();
}

pub fn handle_close_merkle_root_config_err(
    lite_svm: &mut LiteSVM,
    args: HandleCloseMerkleRootConfigArgs,
) -> FailedTransactionMetadata {
    let instructions = handle_close_merkle_root_config_ix(args.clone());
    let HandleCloseMerkleRootConfigArgs { creator, .. } = args;
    let creator_pubkey = creator.pubkey();
    process_transaction(lite_svm, &instructions, Some(&creator_pubkey), &[&creator]).unwrap_err()
}
