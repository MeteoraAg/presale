pub mod helpers;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use anchor_lang::error::ERROR_CODE_OFFSET;
use helpers::*;
use presale::{Escrow, Presale};
use std::rc::Rc;

#[test]
fn test_initialize_escrow_with_invalid_whitelist_mode() {
    let mut setup_context = SetupContext::initialize();
    let mint_0 = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let mint_1 = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let mint_2 = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_authority_fixed_price_presale(
            &mut lite_svm,
            mint_0,
            quote,
            Rc::clone(&user),
        );

    let mut errs = vec![];
    let err_0 = handle_create_permissionless_escrow_err(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );
    errs.push(err_0);

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale(
            &mut lite_svm,
            mint_1,
            quote,
            Rc::clone(&user),
        );
    let err_1 = handle_create_permissionless_escrow_err(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );
    errs.push(err_1);

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fcfs_presale(
            &mut lite_svm,
            mint_2,
            quote,
            Rc::clone(&user),
        );

    let operator = Rc::new(Keypair::new());
    handle_create_operator(
        &mut lite_svm,
        HandleCreateOperatorArgs {
            owner: Rc::clone(&user),
            operator: operator.pubkey(),
        },
    );

    let err_2 = handle_create_permissioned_escrow_with_operator_err(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithOperatorArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            vault_owner: user.pubkey(),
            operator: Rc::clone(&operator),
        },
    );
    errs.push(err_2);

    let expected_err = presale::errors::PresaleError::InvalidPresaleWhitelistMode;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    for err in errs {
        assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
    }
}

#[test]
fn test_initialize_escrow_when_deposit_closed() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            quote,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_create_permissionless_escrow_err(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForDeposit;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_initialize_permissioned_with_authority_escrow_with_invalid_operator() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;
    let user_1 = Rc::new(Keypair::new());
    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1.pubkey(),
        LAMPORTS_PER_SOL,
    );

    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse {
        presale_pubkey: presale_pubkey_0,
        ..
    } = handle_create_predefined_permissioned_with_authority_fixed_price_presale(
        &mut lite_svm,
        mint,
        quote,
        Rc::clone(&user),
    );

    let operator_0 = Rc::new(Keypair::new());
    handle_create_operator(
        &mut lite_svm,
        HandleCreateOperatorArgs {
            owner: Rc::clone(&user),
            operator: operator_0.pubkey(),
        },
    );

    let operator_1 = Rc::new(Keypair::new());
    handle_create_operator(
        &mut lite_svm,
        HandleCreateOperatorArgs {
            owner: Rc::clone(&user_1),
            operator: operator_1.pubkey(),
        },
    );

    let err = handle_create_permissioned_escrow_with_operator_err(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithOperatorArgs {
            presale: presale_pubkey_0,
            owner: Rc::clone(&user),
            vault_owner: user_1.pubkey(),
            operator: Rc::clone(&operator_1),
        },
    );

    let expected_err = presale::errors::PresaleError::InvalidOperator;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_initialize_permissioned_with_merkle_proof_escrow_with_invalid_proof() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale(
            &mut lite_svm,
            mint,
            quote,
            Rc::clone(&user),
        );

    let user_1 = Rc::new(Keypair::new());
    let whitelist_wallet = vec![user_1.pubkey()];

    let merkle_tree = build_merkle_tree(whitelist_wallet);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    let err = handle_create_permissioned_escrow_with_merkle_proof_err(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            proof: merkle_tree.get_node(&user_1.pubkey()).proof.unwrap(),
            merkle_root_config: merkle_tree
                .get_merkle_root_config_pubkey(presale_pubkey, &presale::ID),
        },
    );

    let expected_err = presale::errors::PresaleError::InvalidMerkleProof;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_initialize_permissioned_with_authority_escrow() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_authority_fixed_price_presale(
            &mut lite_svm,
            mint,
            quote,
            Rc::clone(&user),
        );

    let operator = Rc::new(Keypair::new());
    handle_create_operator(
        &mut lite_svm,
        HandleCreateOperatorArgs {
            owner: Rc::clone(&user),
            operator: operator.pubkey(),
        },
    );

    handle_create_permissioned_escrow_with_operator(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithOperatorArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            vault_owner: user.pubkey(),
            operator: Rc::clone(&operator),
        },
    );
}

#[test]
fn test_initialize_permissioned_with_merkle_proof_escrow() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale(
            &mut lite_svm,
            mint,
            quote,
            Rc::clone(&user),
        );

    let whitelist_wallet = vec![user.pubkey()];

    let merkle_tree = build_merkle_tree(whitelist_wallet);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            proof: merkle_tree.get_node(&user.pubkey()).proof.unwrap(),
            merkle_root_config: merkle_tree
                .get_merkle_root_config_pubkey(presale_pubkey, &presale::ID),
        },
    );
}

#[test]
fn test_initialize_permissionless_escrow() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote = anchor_spl::token::spl_token::native_mint::ID;
    let user_pubkey = user.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            quote,
            Rc::clone(&user),
        );

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale = derive_presale(&mint, &quote, &user_pubkey, &presale::ID);
    let escrow = derive_escrow(&presale, &user_pubkey, &presale::ID);

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
    assert_eq!(escrow_state.presale, presale);
    assert_eq!(escrow_state.owner, user_pubkey);
    assert!(escrow_state.created_at > 0);
    assert!(escrow_state.last_refreshed_at > 0);
    assert_eq!(escrow_state.pending_claim_token, 0);

    let presale_state: Presale = lite_svm.get_deserialized_zc_account(&presale).unwrap();
    assert_eq!(presale_state.total_escrow, 1);
}
