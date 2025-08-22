pub mod helpers;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use anchor_lang::error::ERROR_CODE_OFFSET;
use helpers::*;
use presale::{Presale, DEFAULT_PERMISSIONLESS_REGISTRY_INDEX};
use std::rc::Rc;

#[test]
fn test_close_escrow_with_deposit() {
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
            max_amount: presale_state.presale_maximum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let err = handle_close_escrow_err(
        &mut lite_svm,
        HandleCloseEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::EscrowNotEmpty;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_close_escrow_with_unclaimed_token() {
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
            max_amount: presale_state.presale_maximum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    warp_time(
        &mut lite_svm,
        presale_state.vesting_start_time + presale_state.vest_duration / 2,
    );

    let err = handle_close_escrow_err(
        &mut lite_svm,
        HandleCloseEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::EscrowNotEmpty;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_close_escrow_presale_ongoing() {
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

    let user_1 = Rc::new(Keypair::new());
    let funding_amount = LAMPORTS_PER_SOL * 3;
    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1.pubkey(),
        funding_amount,
    );
    wrap_sol(
        &mut lite_svm,
        Rc::clone(&user_1),
        funding_amount - LAMPORTS_PER_SOL,
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount_0 = presale_state.presale_maximum_cap / 2;
    let amount_1 = presale_state.presale_maximum_cap - amount_0;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount_0,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: amount_1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_withdraw(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            amount: amount_0,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_withdraw(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            amount: amount_1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_close_escrow(
        &mut lite_svm,
        HandleCloseEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_close_escrow(
        &mut lite_svm,
        HandleCloseEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert_eq!(presale_state.total_escrow, 0);
}

#[test]
fn test_close_escrow_presale_completed() {
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

    let user_1 = Rc::new(Keypair::new());
    let funding_amount = LAMPORTS_PER_SOL * 3;
    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1.pubkey(),
        funding_amount,
    );
    wrap_sol(
        &mut lite_svm,
        Rc::clone(&user_1),
        funding_amount - LAMPORTS_PER_SOL,
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount_0 = presale_state.presale_maximum_cap / 2;
    let amount_1 = presale_state.presale_maximum_cap - amount_0;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount_0,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: amount_1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.vesting_end_time + 1);

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            refresh_escrow: true,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            refresh_escrow: true,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_close_escrow(
        &mut lite_svm,
        HandleCloseEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_close_escrow(
        &mut lite_svm,
        HandleCloseEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert_eq!(presale_state.total_escrow, 0);

    let presale_registry = presale_state.get_presale_registry(0).unwrap();
    assert_eq!(presale_registry.total_escrow, 0);
}
