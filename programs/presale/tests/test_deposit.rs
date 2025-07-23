pub mod helpers;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use anchor_lang::{error::ERROR_CODE_OFFSET, AccountDeserialize};
use anchor_spl::token_interface::TokenAccount;
use helpers::*;
use presale::{Escrow, Presale};
use std::rc::Rc;

#[test]
fn test_deposit_below_buyer_minimum_cap() {
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

    let deposit_amount = presale_state.buyer_minimum_deposit_cap - 1;

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let err = handle_escrow_deposit_err(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
        },
    );

    let expected_err = presale::errors::PresaleError::DepositAmountOutOfCap;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_deposit_before_presale_start() {
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

    let deposit_amount = LAMPORTS_PER_SOL / 2;

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_start_time - 1);

    let err = handle_escrow_deposit_err(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForDeposit;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_deposit_when_presale_ended() {
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

    let deposit_amount = LAMPORTS_PER_SOL / 2;

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_escrow_deposit_err(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForDeposit;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_deposit() {
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

    let deposit_amount = LAMPORTS_PER_SOL / 2;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
        },
    );

    let escrow = derive_escrow(&presale_pubkey, &user_pubkey, &presale::ID);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(presale_state.total_deposit, deposit_amount);
    assert_eq!(escrow_state.total_deposit, deposit_amount);
}

#[test]
fn test_deposit_with_max_buyer_cap() {
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

    let deposit_amount = 10 * LAMPORTS_PER_SOL;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
        },
    );

    let escrow = derive_escrow(&presale_pubkey, &user_pubkey, &presale::ID);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(
        presale_state.total_deposit,
        presale_state.buyer_maximum_deposit_cap
    );
    assert_eq!(
        escrow_state.total_deposit,
        presale_state.buyer_maximum_deposit_cap
    );
}

#[test]
fn test_deposit_with_max_presale_cap() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let user_1 = Rc::new(Keypair::new());
    let user_1_pubkey = user_1.pubkey();
    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        2 * LAMPORTS_PER_SOL,
    );

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let before_presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let deposit_amount = LAMPORTS_PER_SOL / 2 + 100;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: deposit_amount,
        },
    );

    let after_presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert_eq!(
        after_presale_state.total_deposit,
        after_presale_state.presale_maximum_cap
    );

    // End presale earlier
    assert!(after_presale_state.presale_end_time < before_presale_state.presale_end_time);
    assert!(after_presale_state.lock_start_time < before_presale_state.lock_start_time);
    assert!(after_presale_state.lock_end_time < before_presale_state.lock_end_time);
    assert!(after_presale_state.vesting_start_time < before_presale_state.vesting_start_time);
    assert!(after_presale_state.vesting_end_time < before_presale_state.vesting_end_time);
}

#[test]
fn test_deposit_token2022() {
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
    let user_pubkey = user.pubkey();

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

    let deposit_amount = presale_state.buyer_minimum_deposit_cap + 1;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
        },
    );

    let escrow = derive_escrow(&presale_pubkey, &user_pubkey, &presale::ID);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let quote_token_vault = lite_svm
        .get_account(&presale_state.quote_token_vault)
        .unwrap();

    let quote_token = TokenAccount::try_deserialize(&mut quote_token_vault.data.as_ref()).unwrap();
    assert_eq!(quote_token.amount, deposit_amount);

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(presale_state.total_deposit, deposit_amount);
    assert_eq!(escrow_state.total_deposit, deposit_amount);
}
