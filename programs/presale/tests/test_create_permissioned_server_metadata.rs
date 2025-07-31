use std::rc::Rc;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_lang::error::ERROR_CODE_OFFSET;
use presale::PermissionedServerMetadata;

use crate::helpers::{
    derive_permissioned_server_metadata, handle_close_permissioned_server_metadata,
    handle_create_permissioned_server_metadata, handle_create_permissioned_server_metadata_err,
    handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale,
    handle_create_predefined_permissionless_fixed_price_presale,
    ClosePermissionedServerMetadataArgs, CreatePermissionedServerProofMetadataArgs,
    HandleCreatePredefinedPresaleResponse, LiteSVMExt, SetupContext, DEFAULT_BASE_TOKEN_DECIMALS,
};

pub mod helpers;

#[test]
fn test_create_permissioned_server_metadata() {
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

    handle_create_permissioned_server_metadata(
        &mut lite_svm,
        CreatePermissionedServerProofMetadataArgs {
            presale: presale_pubkey,
            server_url: "https://example.com/proof.json".to_string(),
            owner: Rc::clone(&user),
        },
    );

    let permissioned_server_metadata =
        derive_permissioned_server_metadata(&presale_pubkey, &presale::ID);
    let permissioned_server_metadata_state: PermissionedServerMetadata = lite_svm
        .get_deserialized_account(&permissioned_server_metadata)
        .unwrap();

    assert_eq!(permissioned_server_metadata_state.presale, presale_pubkey);
    assert_eq!(
        permissioned_server_metadata_state.server_url,
        "https://example.com/proof.json"
    );
}

#[test]
fn test_create_permissioned_server_metadata_with_permissionless_presale() {
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

    let err = handle_create_permissioned_server_metadata_err(
        &mut lite_svm,
        CreatePermissionedServerProofMetadataArgs {
            presale: presale_pubkey,
            server_url: "https://example.com/proof.json".to_string(),
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::InvalidPresaleWhitelistMode;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_close_permissioned_server_metadata() {
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

    handle_create_permissioned_server_metadata(
        &mut lite_svm,
        CreatePermissionedServerProofMetadataArgs {
            presale: presale_pubkey,
            server_url: "https://example.com/proof.json".to_string(),
            owner: Rc::clone(&user),
        },
    );

    handle_close_permissioned_server_metadata(
        &mut lite_svm,
        ClosePermissionedServerMetadataArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let permissioned_server_metadata =
        derive_permissioned_server_metadata(&presale_pubkey, &presale::ID);
    let permissioned_server_metadata_account =
        lite_svm.get_account(&permissioned_server_metadata).unwrap();

    assert_eq!(
        permissioned_server_metadata_account.owner,
        Pubkey::default() // Account should be closed, so owner is default
    );
}
