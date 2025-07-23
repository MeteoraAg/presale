pub mod helpers;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use anchor_lang::error::ERROR_CODE_OFFSET;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_interface::TokenAccount,
};
use helpers::*;
use presale::{Escrow, Presale};
use std::rc::Rc;

#[test]
fn test_withdraw_remaining_quote_failed_fixed_price_presale() {
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

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap - 1,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let user_quote_token_pubkey = get_associated_token_address_with_program_id(
        &user_pubkey,
        &presale_state.quote_mint,
        &anchor_spl::token::ID,
    );

    let before_user_quote_token: TokenAccount = lite_svm
        .get_deserialized_account(&user_quote_token_pubkey)
        .unwrap();

    handle_escrow_withdraw_remaining_quote(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_user_quote_token: TokenAccount = lite_svm
        .get_deserialized_account(&user_quote_token_pubkey)
        .unwrap();

    let escrow = derive_escrow(&presale_pubkey, &user_pubkey, &presale::ID);
    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(escrow_state.is_remaining_quote_withdrawn, 1);

    assert_eq!(
        after_user_quote_token.amount - before_user_quote_token.amount,
        escrow_state.total_deposit
    );
}

#[test]
fn test_withdraw_remaining_quote_failed_fcfs_presale() {
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

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap - 1,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let user_quote_token_pubkey = get_associated_token_address_with_program_id(
        &user_pubkey,
        &presale_state.quote_mint,
        &anchor_spl::token::ID,
    );

    let before_user_quote_token: TokenAccount = lite_svm
        .get_deserialized_account(&user_quote_token_pubkey)
        .unwrap();

    handle_escrow_withdraw_remaining_quote(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_user_quote_token: TokenAccount = lite_svm
        .get_deserialized_account(&user_quote_token_pubkey)
        .unwrap();

    let escrow = derive_escrow(&presale_pubkey, &user_pubkey, &presale::ID);
    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(escrow_state.is_remaining_quote_withdrawn, 1);

    assert_eq!(
        after_user_quote_token.amount - before_user_quote_token.amount,
        escrow_state.total_deposit
    );
}

#[test]
fn test_withdraw_remaining_quote_failed_prorata_presale() {
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

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap - 1,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let user_quote_token_pubkey = get_associated_token_address_with_program_id(
        &user_pubkey,
        &presale_state.quote_mint,
        &anchor_spl::token::ID,
    );

    let before_user_quote_token: TokenAccount = lite_svm
        .get_deserialized_account(&user_quote_token_pubkey)
        .unwrap();

    handle_escrow_withdraw_remaining_quote(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_user_quote_token: TokenAccount = lite_svm
        .get_deserialized_account(&user_quote_token_pubkey)
        .unwrap();

    let escrow = derive_escrow(&presale_pubkey, &user_pubkey, &presale::ID);
    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(escrow_state.is_remaining_quote_withdrawn, 1);

    assert_eq!(
        after_user_quote_token.amount - before_user_quote_token.amount,
        escrow_state.total_deposit
    );
}

#[test]
fn test_withdraw_remaining_quote_success_prorata_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    let user_1 = Rc::new(Keypair::new());
    let user_1_pubkey = user_1.pubkey();

    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        2 * LAMPORTS_PER_SOL,
    );

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: LAMPORTS_PER_SOL,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: LAMPORTS_PER_SOL,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let user_quote_token_pubkey = get_associated_token_address_with_program_id(
        &user_pubkey,
        &presale_state.quote_mint,
        &anchor_spl::token::ID,
    );

    let before_user_quote_token: TokenAccount = lite_svm
        .get_deserialized_account(&user_quote_token_pubkey)
        .unwrap();

    handle_escrow_withdraw_remaining_quote(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_user_quote_token: TokenAccount = lite_svm
        .get_deserialized_account(&user_quote_token_pubkey)
        .unwrap();

    let escrow = derive_escrow(&presale_pubkey, &user_pubkey, &presale::ID);
    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(escrow_state.is_remaining_quote_withdrawn, 1);

    let refund_amount = (presale_state.total_deposit - presale_state.presale_maximum_cap)
        * escrow_state.total_deposit
        / presale_state.total_deposit;

    assert_eq!(
        after_user_quote_token.amount - before_user_quote_token.amount,
        refund_amount
    );
}

#[test]
fn test_withdraw_remaining_quote_success_fixed_price_presale() {
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

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap + 10,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_escrow_withdraw_remaining_quote_err(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForWithdrawRemainingQuote;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_withdraw_remaining_quote_success_fcfs_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fcfs_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap + 10,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_escrow_withdraw_remaining_quote_err(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForWithdrawRemainingQuote;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_withdraw_remaining_quote_when_presale_ongoing() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap - 1,
        },
    );

    let err = handle_escrow_withdraw_remaining_quote_err(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForWithdrawRemainingQuote;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}
