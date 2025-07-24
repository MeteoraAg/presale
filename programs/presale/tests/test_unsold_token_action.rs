pub mod helpers;

use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::{error::ERROR_CODE_OFFSET, AccountDeserialize};
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_interface::TokenAccount,
};
use helpers::*;
use presale::{Presale, UnsoldTokenAction, WhitelistMode};
use std::rc::Rc;

#[test]
fn test_unsold_token_action_fcfs_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fcfs_presale(
            &mut lite_svm,
            mint,
            quote_mint,
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
            max_amount: presale_state.presale_minimum_cap + 1,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_perform_unsold_token_action_err(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::NoUnsoldBaseTokens;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_unsold_token_action_prorata_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale(
            &mut lite_svm,
            mint,
            quote_mint,
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
            max_amount: presale_state.presale_minimum_cap + 1,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_perform_unsold_token_action_err(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::NoUnsoldBaseTokens;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_unsold_token_action_refund_fixed_price_presale_before_presale_complete() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let presale_pubkey = derive_presale(&mint, &quote_mint, &user.pubkey(), &presale::ID);

    let init_fp_presale_ix = create_predefined_fixed_price_presale_ix(
        &mut lite_svm,
        mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
        UnsoldTokenAction::Refund,
    );

    process_transaction(
        &mut lite_svm,
        &init_fp_presale_ix,
        Some(&user.pubkey()),
        &[&user],
    )
    .unwrap();

    let err = handle_perform_unsold_token_action_err(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotCompleted;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_unsold_token_action_refund_fixed_price_presale_when_presale_failed() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let presale_pubkey = derive_presale(&mint, &quote_mint, &user.pubkey(), &presale::ID);

    let init_fp_presale_ix = create_predefined_fixed_price_presale_ix(
        &mut lite_svm,
        mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
        UnsoldTokenAction::Refund,
    );

    process_transaction(
        &mut lite_svm,
        &init_fp_presale_ix,
        Some(&user.pubkey()),
        &[&user],
    )
    .unwrap();

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_perform_unsold_token_action_err(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotCompleted;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_unsold_token_action_refund_fixed_price_presale_token2022() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let presale_pubkey = derive_presale(&mint, &quote_mint, &user.pubkey(), &presale::ID);

    let init_fp_presale_ix = create_predefined_fixed_price_presale_ix(
        &mut lite_svm,
        mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
        UnsoldTokenAction::Refund,
    );

    process_transaction(
        &mut lite_svm,
        &init_fp_presale_ix,
        Some(&user.pubkey()),
        &[&user],
    )
    .unwrap();

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap + 1,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let creator_base_token = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &presale_state.base_mint,
        &get_program_id_from_token_flag(presale_state.base_token_program_flag),
    );

    let before_creator_base_token = lite_svm.get_account(&creator_base_token).unwrap();

    let before_base_token_vault = lite_svm
        .get_account(&presale_state.base_token_vault)
        .unwrap();

    handle_perform_unsold_token_action(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let after_creator_base_token = lite_svm.get_account(&creator_base_token).unwrap();

    let after_base_token_vault = lite_svm
        .get_account(&presale_state.base_token_vault)
        .unwrap();

    let before_base_token_balance =
        TokenAccount::try_deserialize(&mut before_base_token_vault.data.as_ref())
            .unwrap()
            .amount;

    let after_base_token_balance =
        TokenAccount::try_deserialize(&mut after_base_token_vault.data.as_ref())
            .unwrap()
            .amount;

    assert!(after_base_token_balance < before_base_token_balance);

    let before_creator_base_token_balance =
        TokenAccount::try_deserialize(&mut before_creator_base_token.data.as_ref())
            .unwrap()
            .amount;

    let after_creator_base_token_balance =
        TokenAccount::try_deserialize(&mut after_creator_base_token.data.as_ref())
            .unwrap()
            .amount;

    assert!(after_creator_base_token_balance > before_creator_base_token_balance);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert!(presale_state.is_fixed_price_presale_unsold_token_action_performed == 1);
}

#[test]
fn test_unsold_token_action_refund_fixed_price_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let presale_pubkey = derive_presale(&mint, &quote_mint, &user.pubkey(), &presale::ID);

    let init_fp_presale_ix = create_predefined_fixed_price_presale_ix(
        &mut lite_svm,
        mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
        UnsoldTokenAction::Refund,
    );

    process_transaction(
        &mut lite_svm,
        &init_fp_presale_ix,
        Some(&user.pubkey()),
        &[&user],
    )
    .unwrap();

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap + 1,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let creator_base_token = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &presale_state.base_mint,
        &get_program_id_from_token_flag(presale_state.base_token_program_flag),
    );

    let before_creator_base_token = lite_svm.get_account(&creator_base_token).unwrap();

    let before_base_token_vault = lite_svm
        .get_account(&presale_state.base_token_vault)
        .unwrap();

    handle_perform_unsold_token_action(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let after_creator_base_token = lite_svm.get_account(&creator_base_token).unwrap();

    let after_base_token_vault = lite_svm
        .get_account(&presale_state.base_token_vault)
        .unwrap();

    let before_base_token_balance =
        TokenAccount::try_deserialize(&mut before_base_token_vault.data.as_ref())
            .unwrap()
            .amount;

    let after_base_token_balance =
        TokenAccount::try_deserialize(&mut after_base_token_vault.data.as_ref())
            .unwrap()
            .amount;

    assert!(after_base_token_balance < before_base_token_balance);

    let before_creator_base_token_balance =
        TokenAccount::try_deserialize(&mut before_creator_base_token.data.as_ref())
            .unwrap()
            .amount;

    let after_creator_base_token_balance =
        TokenAccount::try_deserialize(&mut after_creator_base_token.data.as_ref())
            .unwrap()
            .amount;

    assert!(after_creator_base_token_balance > before_creator_base_token_balance);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert!(presale_state.is_fixed_price_presale_unsold_token_action_performed == 1);
}

#[test]
fn test_unsold_token_action_burn_fixed_price_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let presale_pubkey = derive_presale(&mint, &quote_mint, &user.pubkey(), &presale::ID);

    let init_fp_presale_ix = create_predefined_fixed_price_presale_ix(
        &mut lite_svm,
        mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
        UnsoldTokenAction::Burn,
    );

    process_transaction(
        &mut lite_svm,
        &init_fp_presale_ix,
        Some(&user.pubkey()),
        &[&user],
    )
    .unwrap();

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap + 1,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let before_base_token_vault = lite_svm
        .get_account(&presale_state.base_token_vault)
        .unwrap();

    handle_perform_unsold_token_action(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let after_base_token_vault = lite_svm
        .get_account(&presale_state.base_token_vault)
        .unwrap();

    let before_base_token_balance =
        TokenAccount::try_deserialize(&mut before_base_token_vault.data.as_ref())
            .unwrap()
            .amount;

    let after_base_token_balance =
        TokenAccount::try_deserialize(&mut after_base_token_vault.data.as_ref())
            .unwrap()
            .amount;

    assert!(after_base_token_balance < before_base_token_balance);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert!(presale_state.is_fixed_price_presale_unsold_token_action_performed == 1);

    let err = handle_perform_unsold_token_action_err(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::UnsoldBaseTokenActionAlreadyPerformed;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}
