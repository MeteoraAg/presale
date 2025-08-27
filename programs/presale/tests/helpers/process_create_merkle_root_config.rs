use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::{pubkey::Pubkey, signature::Keypair};
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use merkle_tree::config_merkle_tree::ConfigMerkleTree;
use merkle_tree::tree_node::TreeNode;
use presale::CreateMerkleRootConfigParams;
use std::rc::Rc;

use crate::helpers::{derive_event_authority, derive_merkle_root_config, process_transaction};

#[derive(Debug, Clone)]
pub struct WhitelistWallet {
    pub address: Pubkey,
    pub registry_index: u8,
    pub max_deposit_cap: u64,
}

pub fn build_merkle_tree(
    whitelist_wallets: Vec<WhitelistWallet>,
    version: u64,
) -> ConfigMerkleTree {
    let tree_nodes = whitelist_wallets
        .into_iter()
        .map(|wallet| TreeNode {
            escrow_owner: wallet.address,
            registry_index: wallet.registry_index,
            proof: None,
            deposit_cap: wallet.max_deposit_cap,
        })
        .collect::<Vec<_>>();

    ConfigMerkleTree::new(tree_nodes, version).unwrap()
}

pub struct HandleCreateMerkleRootConfigArgs<'a> {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
    pub merkle_tree: &'a ConfigMerkleTree,
}

pub fn handle_create_merkle_root_config_ix(args: HandleCreateMerkleRootConfigArgs) -> Instruction {
    let HandleCreateMerkleRootConfigArgs {
        presale,
        owner,
        merkle_tree,
    } = args;

    let owner_pubkey = owner.pubkey();
    let config_merkle_root = derive_merkle_root_config(&presale, merkle_tree.version, &presale::ID);

    let ix_data = presale::instruction::CreateMerkleRootConfig {
        params: CreateMerkleRootConfigParams {
            root: merkle_tree.merkle_root,
            version: merkle_tree.version,
        },
    }
    .data();

    let accounts = presale::accounts::CreateMerkleRootConfigCtx {
        merkle_root_config: config_merkle_root,
        creator: owner_pubkey,
        system_program: anchor_lang::solana_program::system_program::ID,
        program: presale::ID,
        presale,
        event_authority: derive_event_authority(&presale::ID),
    }
    .to_account_metas(None);

    Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    }
}

pub fn handle_create_merkle_root_config(
    lite_svm: &mut LiteSVM,
    args: HandleCreateMerkleRootConfigArgs,
) {
    let merkle_tree = &args.merkle_tree;

    let config_merkle_root =
        derive_merkle_root_config(&args.presale, merkle_tree.version, &presale::ID);
    let config_account = lite_svm.get_account(&config_merkle_root);

    if config_account.is_some() {
        return; // Merkle root config account already exists
    }

    let instruction = handle_create_merkle_root_config_ix(HandleCreateMerkleRootConfigArgs {
        presale: args.presale,
        owner: Rc::clone(&args.owner),
        merkle_tree,
    });

    let HandleCreateMerkleRootConfigArgs { owner, .. } = args;

    process_transaction(lite_svm, &[instruction], Some(&owner.pubkey()), &[&owner]).unwrap();
}
