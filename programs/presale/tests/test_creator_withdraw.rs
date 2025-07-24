pub mod helpers;

use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::error::ERROR_CODE_OFFSET;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_interface::TokenAccount,
};
use helpers::*;
use presale::Presale;
use std::rc::Rc;

#[test]
fn test_creator_withdraw_presale_token_2022_success() {
    let mut setup_context = SetupContext::initialize();
    let base_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let quote_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_QUOTE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_QUOTE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            base_mint,
            quote_mint,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount = presale_state.presale_minimum_cap;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let creator_quote_token_address = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &presale_state.quote_mint,
        &get_program_id_from_token_flag(presale_state.quote_token_program_flag),
    );

    let before_creator_quote_account: TokenAccount = lite_svm
        .get_deserialized_account(&creator_quote_token_address)
        .unwrap();

    handle_creator_withdraw_token(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_creator_quote_account: TokenAccount = lite_svm
        .get_deserialized_account(&creator_quote_token_address)
        .unwrap();

    assert!(after_creator_quote_account.amount > before_creator_quote_account.amount);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert_eq!(presale_state.has_creator_withdrawn, 1);
}

#[test]
fn test_creator_withdraw_presale_token_2022_failed() {
    let mut setup_context = SetupContext::initialize();
    let base_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let quote_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_QUOTE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_QUOTE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            base_mint,
            quote_mint,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount = presale_state.presale_minimum_cap - 1;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let creator_base_token_address = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &presale_state.base_mint,
        &get_program_id_from_token_flag(presale_state.base_token_program_flag),
    );

    let before_creator_base_account: TokenAccount = lite_svm
        .get_deserialized_account(&creator_base_token_address)
        .unwrap();

    handle_creator_withdraw_token(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_creator_base_account: TokenAccount = lite_svm
        .get_deserialized_account(&creator_base_token_address)
        .unwrap();

    assert!(after_creator_base_account.amount > before_creator_base_account.amount);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert_eq!(presale_state.has_creator_withdrawn, 1);
}

#[test]
fn test_creator_withdraw_presale_success() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount = presale_state.presale_minimum_cap;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let creator_quote_token_address = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &presale_state.quote_mint,
        &get_program_id_from_token_flag(presale_state.quote_token_program_flag),
    );

    let before_creator_quote_account: TokenAccount = lite_svm
        .get_deserialized_account(&creator_quote_token_address)
        .unwrap();

    handle_creator_withdraw_token(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_creator_quote_account: TokenAccount = lite_svm
        .get_deserialized_account(&creator_quote_token_address)
        .unwrap();

    assert!(after_creator_quote_account.amount > before_creator_quote_account.amount);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert_eq!(presale_state.has_creator_withdrawn, 1);
}

#[test]
fn test_creator_withdraw_presale_failed() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount = presale_state.presale_minimum_cap - 1;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let creator_base_token_address = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &presale_state.base_mint,
        &get_program_id_from_token_flag(presale_state.base_token_program_flag),
    );

    let before_creator_base_account: TokenAccount = lite_svm
        .get_deserialized_account(&creator_base_token_address)
        .unwrap();

    handle_creator_withdraw_token(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_creator_base_account: TokenAccount = lite_svm
        .get_deserialized_account(&creator_base_token_address)
        .unwrap();

    assert!(after_creator_base_account.amount > before_creator_base_account.amount);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert_eq!(presale_state.has_creator_withdrawn, 1);
}

#[test]
fn test_creator_withdraw_when_presale_ongoing() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let err = handle_creator_withdraw_token_err(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForWithdraw;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}
