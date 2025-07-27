use std::rc::Rc;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_lang::error::ERROR_CODE_OFFSET;
use presale::MerkleProofMetadata;

use crate::helpers::{
    derive_merkle_proof_metadata, handle_close_merkle_proof_metadata,
    handle_create_merkle_proof_metadata, handle_create_merkle_proof_metadata_err,
    handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale,
    handle_create_predefined_permissionless_fixed_price_presale, CloseMerkleProofMetadataArgs,
    CreateMerkleProofMetadataArgs, HandleCreatePredefinedPresaleResponse, LiteSVMExt, SetupContext,
    DEFAULT_BASE_TOKEN_DECIMALS,
};

pub mod helpers;

#[test]
fn test_create_merkle_proof_metadata() {
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

    handle_create_merkle_proof_metadata(
        &mut lite_svm,
        CreateMerkleProofMetadataArgs {
            presale: presale_pubkey,
            proof_url: "https://example.com/proof.json".to_string(),
            owner: Rc::clone(&user),
        },
    );

    let merkle_proof_metadata = derive_merkle_proof_metadata(&presale_pubkey, &presale::ID);
    let merkle_proof_metadata_state: MerkleProofMetadata = lite_svm
        .get_deserialized_account(&merkle_proof_metadata)
        .unwrap();

    assert_eq!(merkle_proof_metadata_state.presale, presale_pubkey);
    assert_eq!(
        merkle_proof_metadata_state.proof_url,
        "https://example.com/proof.json"
    );
}

#[test]
fn test_create_merkle_proof_metadata_with_permissionless_presale() {
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

    let err = handle_create_merkle_proof_metadata_err(
        &mut lite_svm,
        CreateMerkleProofMetadataArgs {
            presale: presale_pubkey,
            proof_url: "https://example.com/proof.json".to_string(),
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::InvalidPresaleWhitelistMode;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_close_merkle_proof_metadata() {
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

    handle_create_merkle_proof_metadata(
        &mut lite_svm,
        CreateMerkleProofMetadataArgs {
            presale: presale_pubkey,
            proof_url: "https://example.com/proof.json".to_string(),
            owner: Rc::clone(&user),
        },
    );

    handle_close_merkle_proof_metadata(
        &mut lite_svm,
        CloseMerkleProofMetadataArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let merkle_proof_metadata = derive_merkle_proof_metadata(&presale_pubkey, &presale::ID);
    let merkle_proof_metadata_account = lite_svm.get_account(&merkle_proof_metadata).unwrap();

    assert_eq!(
        merkle_proof_metadata_account.owner,
        Pubkey::default() // Account should be closed, so owner is default
    );
}
