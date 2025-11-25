pub mod helpers;
use std::rc::Rc;

use anchor_client::solana_sdk::{instruction::Instruction, signature::Keypair, signer::Signer};
use anchor_lang::error::ERROR_CODE_OFFSET;
use anchor_lang::prelude::{Clock, Pubkey};
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use anchor_spl::token::spl_token::instruction::mint_to;
use helpers::*;
use litesvm::LiteSVM;
use merkle_tree::config_merkle_tree::ConfigMerkleTree;
use presale::{Presale, WhitelistMode, DEFAULT_PERMISSIONLESS_REGISTRY_INDEX};

struct SetupResponse {
    pub presale_pubkey: Pubkey,
    pub lite_svm: LiteSVM,
    pub user: Rc<Keypair>,
    pub user_1: Rc<Keypair>,
    pub merkle_tree: ConfigMerkleTree,
}

fn setup() -> SetupResponse {
    let mut setup_context = SetupContext::initialize();
    let user_1 = setup_context.create_user();

    let mint_0 = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let CreateDefaultFixedPricePresaleArgsWrapper {
        mut presale_params_wrapper,
        fixed_point_params_wrapper,
    } = create_default_fixed_price_presale_args_wrapper(
        mint_0,
        quote,
        &lite_svm,
        WhitelistMode::PermissionWithMerkleProof,
        Rc::clone(&user),
        user_pubkey,
    );

    let presale_pubkey = presale_params_wrapper.accounts.presale;
    let version = 0;

    let clock: Clock = lite_svm.get_sysvar();
    presale_params_wrapper
        .args
        .params
        .presale_params
        .presale_start_time = (clock.unix_timestamp + 1) as u64;

    let init_fp_ix = Instruction {
        accounts: fixed_point_params_wrapper.accounts.to_account_metas(None),
        data: fixed_point_params_wrapper.args.data(),
        program_id: presale::ID,
    };

    let init_presale_ix = presale_params_wrapper.to_instructions();
    let mut instructions = vec![init_fp_ix];
    instructions.extend_from_slice(&init_presale_ix);

    process_transaction(&mut lite_svm, &instructions, Some(&user_pubkey), &[&user]).unwrap();

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let whitelist_wallet = vec![WhitelistWallet {
        address: user_pubkey,
        registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        max_deposit_cap: presale_state.presale_maximum_cap,
    }];

    let merkle_tree = build_merkle_tree(whitelist_wallet, version);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    SetupResponse {
        presale_pubkey,
        lite_svm,
        user,
        user_1,
        merkle_tree,
    }
}

#[test]
fn test_close_merkle_root_config_when_presale_ongoing() {
    let SetupResponse {
        presale_pubkey,
        mut lite_svm,
        user,
        merkle_tree,
        ..
    } = setup();

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_start_time);

    let err = handle_close_merkle_root_config_err(
        &mut lite_svm,
        HandleCloseMerkleRootConfigArgs {
            presale: presale_pubkey,
            version: merkle_tree.version as u8,
            creator: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleOngoing;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_close_merkle_root_config_invalid_creator() {
    let SetupResponse {
        presale_pubkey,
        mut lite_svm,
        user_1,
        merkle_tree,
        ..
    } = setup();

    let err = handle_close_merkle_root_config_err(
        &mut lite_svm,
        HandleCloseMerkleRootConfigArgs {
            presale: presale_pubkey,
            version: merkle_tree.version as u8,
            creator: Rc::clone(&user_1),
        },
    );

    let expected_err = presale::errors::PresaleError::InvalidCreatorAccount;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_close_merkle_roof_config_invalid_presale() {
    let SetupResponse {
        presale_pubkey,
        mut lite_svm,
        user_1,
        user,
        merkle_tree,
        ..
    } = setup();

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    let base_mint_account = lite_svm.get_account(&presale_state.base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&presale_state.quote_mint).unwrap();

    let user_pubkey = user.pubkey();
    let user_1_pubkey = user_1.pubkey();

    let create_base_ata_ix = create_associated_token_account_idempotent(
        &user_1_pubkey,
        &user_1_pubkey,
        &presale_state.base_mint,
        &base_mint_account.owner,
    );

    let create_quote_ata_ix = create_associated_token_account_idempotent(
        &user_1_pubkey,
        &user_1_pubkey,
        &presale_state.quote_mint,
        &quote_mint_account.owner,
    );

    let user_1_base_ata = get_associated_token_address_with_program_id(
        &user_1_pubkey,
        &presale_state.base_mint,
        &base_mint_account.owner,
    );

    let mint_ix = mint_to(
        &base_mint_account.owner,
        &presale_state.base_mint,
        &user_1_base_ata,
        &user_pubkey,
        &[],
        1_000_000_000_000_000,
    )
    .unwrap();

    let ixs = vec![create_base_ata_ix, create_quote_ata_ix, mint_ix];
    process_transaction(&mut lite_svm, &ixs, Some(&user_1_pubkey), &[&user, &user_1]).unwrap();

    let HandleCreatePredefinedPresaleResponse {
        presale_pubkey: new_presale_pubkey,
        ..
    } = handle_create_predefined_permissionless_fixed_price_presale(
        &mut lite_svm,
        presale_state.base_mint,
        presale_state.quote_mint,
        Rc::clone(&user_1),
    );

    let HandleCloseMerkleRootConfigWrapper {
        instructions,
        mut accounts,
    } = handle_close_merkle_root_config_wrapper(HandleCloseMerkleRootConfigArgs {
        presale: presale_pubkey,
        version: merkle_tree.version as u8,
        creator: Rc::clone(&user),
    });

    accounts.presale = new_presale_pubkey;

    let instruction = Instruction {
        program_id: presale::ID,
        accounts: accounts.to_account_metas(None),
        data: instructions.data(),
    };

    let err = process_transaction(&mut lite_svm, &[instruction], Some(&user_pubkey), &[&user])
        .unwrap_err();

    let error_code = anchor_lang::error::ErrorCode::ConstraintHasOne;
    let err_str = format!("Error Number: {}.", error_code as u32);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_close_merkle_root_config_presale_not_started_success() {
    let SetupResponse {
        presale_pubkey,
        mut lite_svm,
        user,
        merkle_tree,
        ..
    } = setup();

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    let clock: Clock = lite_svm.get_sysvar();
    assert!(
        presale_state.get_presale_progress(clock.unix_timestamp as u64)
            == presale::PresaleProgress::NotStarted
    );

    handle_close_merkle_root_config(
        &mut lite_svm,
        HandleCloseMerkleRootConfigArgs {
            presale: presale_pubkey,
            version: merkle_tree.version as u8,
            creator: Rc::clone(&user),
        },
    );

    let merkle_root_config =
        derive_merkle_root_config(&presale_pubkey, merkle_tree.version, &presale::ID);

    let merkle_root_account = lite_svm.get_account(&merkle_root_config).unwrap();
    assert!(merkle_root_account.data.is_empty());
    assert_eq!(merkle_root_account.owner, Pubkey::default());
}

#[test]
fn test_close_merkle_root_config_presale_failed_success() {
    let SetupResponse {
        presale_pubkey,
        mut lite_svm,
        user,
        merkle_tree,
        ..
    } = setup();

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time);
    let clock: Clock = lite_svm.get_sysvar();

    assert!(
        presale_state.get_presale_progress(clock.unix_timestamp as u64)
            == presale::PresaleProgress::Failed
    );

    handle_close_merkle_root_config(
        &mut lite_svm,
        HandleCloseMerkleRootConfigArgs {
            presale: presale_pubkey,
            version: merkle_tree.version as u8,
            creator: Rc::clone(&user),
        },
    );

    let merkle_root_config =
        derive_merkle_root_config(&presale_pubkey, merkle_tree.version, &presale::ID);

    let merkle_root_account = lite_svm.get_account(&merkle_root_config).unwrap();
    assert!(merkle_root_account.data.is_empty());
    assert_eq!(merkle_root_account.owner, Pubkey::default());
}

#[test]
fn test_close_merkle_root_config_presale_completed_success() {
    let SetupResponse {
        presale_pubkey,
        mut lite_svm,
        user,
        merkle_tree,
        ..
    } = setup();

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time);
    let clock: Clock = lite_svm.get_sysvar();

    assert!(
        presale_state.get_presale_progress(clock.unix_timestamp as u64)
            == presale::PresaleProgress::Failed
    );

    handle_close_merkle_root_config(
        &mut lite_svm,
        HandleCloseMerkleRootConfigArgs {
            presale: presale_pubkey,
            version: merkle_tree.version as u8,
            creator: Rc::clone(&user),
        },
    );

    let merkle_root_config =
        derive_merkle_root_config(&presale_pubkey, merkle_tree.version, &presale::ID);

    let merkle_root_account = lite_svm.get_account(&merkle_root_config).unwrap();
    assert!(merkle_root_account.data.is_empty());
    assert_eq!(merkle_root_account.owner, Pubkey::default());
}

#[test]
fn test_close_merkle_root_config_presale_success() {
    let SetupResponse {
        presale_pubkey,
        mut lite_svm,
        merkle_tree,
        user,
        ..
    } = setup();

    let user_pubkey = user.pubkey();

    let merkle_root_config =
        merkle_tree.get_merkle_root_config_pubkey(presale_pubkey, &presale::ID);

    let tree_node = merkle_tree.get_node(&user_pubkey);
    let proof = tree_node.proof.unwrap();

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_start_time);

    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_root_config,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            max_deposit_cap: tree_node.deposit_cap,
            proof,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: tree_node.deposit_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    let clock: Clock = lite_svm.get_sysvar();

    assert!(
        presale_state.get_presale_progress(clock.unix_timestamp as u64)
            == presale::PresaleProgress::Completed
    );

    handle_close_merkle_root_config(
        &mut lite_svm,
        HandleCloseMerkleRootConfigArgs {
            presale: presale_pubkey,
            version: merkle_tree.version as u8,
            creator: Rc::clone(&user),
        },
    );

    let merkle_root_config =
        derive_merkle_root_config(&presale_pubkey, merkle_tree.version, &presale::ID);

    let merkle_root_account = lite_svm.get_account(&merkle_root_config).unwrap();
    assert!(merkle_root_account.data.is_empty());
    assert_eq!(merkle_root_account.owner, Pubkey::default());
}
