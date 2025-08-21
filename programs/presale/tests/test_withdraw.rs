pub mod helpers;

use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::error::ERROR_CODE_OFFSET;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_interface::TokenAccount,
};
use helpers::*;
use presale::{Escrow, Presale, DEFAULT_PERMISSIONLESS_REGISTRY_INDEX};
use std::rc::Rc;

#[test]
fn test_withdraw_when_presale_end() {
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

    let deposit_amount = 1_000_000;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_escrow_withdraw_err(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForWithdraw;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_withdraw_over_escrow_balance() {
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

    let deposit_amount = 1_000_000;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let err = handle_escrow_withdraw_err(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            amount: deposit_amount + 1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::InsufficientEscrowBalance;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_withdraw_fixed_price_presale_token2022() {
    let mut setup_context = SetupContext::initialize();
    let base_mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let quote_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_QUOTE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_QUOTE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            base_mint,
            quote_mint,
            Rc::clone(&user),
        );

    let deposit_amount = 1_000_000;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let escrow = derive_escrow(
        &presale_pubkey,
        &user_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );
    let user_quote_token = get_associated_token_address_with_program_id(
        &user_pubkey,
        &quote_mint,
        &anchor_spl::token_2022::spl_token_2022::ID,
    );

    let before_user_quote_token_account = lite_svm
        .get_deserialized_account::<TokenAccount>(&user_quote_token)
        .unwrap();

    let before_escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    let before_presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    let withdraw_amount = before_escrow_state.total_deposit / 2;

    handle_escrow_withdraw(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            amount: withdraw_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let after_escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    let after_presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    let after_user_quote_token_account = lite_svm
        .get_deserialized_account::<TokenAccount>(&user_quote_token)
        .unwrap();

    let withdrawn_amount = before_escrow_state.total_deposit - after_escrow_state.total_deposit;
    assert_eq!(withdrawn_amount, withdraw_amount);
    assert_eq!(
        after_presale_state.total_deposit,
        before_presale_state.total_deposit - withdrawn_amount
    );

    let actual_received_amount =
        after_user_quote_token_account.amount - before_user_quote_token_account.amount;

    // transfer fee
    assert!(actual_received_amount < withdraw_amount);
}

#[test]
fn test_withdraw_fixed_price_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let deposit_amount = 1_000_000;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let escrow = derive_escrow(
        &presale_pubkey,
        &user_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );
    let before_escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    let before_presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    let withdraw_amount = before_escrow_state.total_deposit / 2;

    handle_escrow_withdraw(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            amount: withdraw_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let after_escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    let after_presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    let withdrawn_amount = before_escrow_state.total_deposit - after_escrow_state.total_deposit;
    assert_eq!(withdrawn_amount, withdraw_amount);
    assert_eq!(
        after_presale_state.total_deposit,
        before_presale_state.total_deposit - withdrawn_amount
    );
}

#[test]
fn test_withdraw_prorata_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let deposit_amount = 1_000_000;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let escrow = derive_escrow(
        &presale_pubkey,
        &user_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );
    let before_escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    let withdraw_amount = before_escrow_state.total_deposit / 2;

    handle_escrow_withdraw(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            amount: withdraw_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let after_escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    let withdrawn_amount = before_escrow_state.total_deposit - after_escrow_state.total_deposit;
    assert_eq!(withdrawn_amount, withdraw_amount);
}

#[test]
fn test_withdraw_fcfs_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fcfs_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let deposit_amount = 1_000_000;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let escrow = derive_escrow(
        &presale_pubkey,
        &user_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );
    let before_escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    let withdraw_amount = before_escrow_state.total_deposit / 2;

    let err = handle_escrow_withdraw_err(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            amount: withdraw_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForWithdraw;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}
