pub mod helpers;

use anchor_lang::{
    error::ERROR_CODE_OFFSET,
    prelude::{AccountMeta, Clock, Pubkey},
};
use anchor_spl::token_interface::TokenAccount;
use helpers::*;
use litesvm::LiteSVM;
use std::rc::Rc;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use presale::{
    LockedVestingArgs, Presale, PresaleArgs, PresaleMode, TokenomicArgs, UnsoldTokenAction,
    WhitelistMode, MAXIMUM_DURATION_UNTIL_PRESALE, MAXIMUM_LOCK_AND_VEST_DURATION,
    MAXIMUM_PRESALE_DURATION, MINIMUM_PRESALE_DURATION,
};

fn assert_err_invalid_tokenomic(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
    mint: Pubkey,
    quote_mint: Pubkey,
    mut tokenomic: TokenomicArgs,
    presale_params: PresaleArgs,
    locked_vesting_params: LockedVestingArgs,
) {
    tokenomic.presale_pool_supply = 0;

    let err = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    let expected_err = presale::errors::PresaleError::InvalidTokenSupply;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

fn assert_err_invalid_locked_vesting_param(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
    mint: Pubkey,
    quote_mint: Pubkey,
    tokenomic: TokenomicArgs,
    presale_params: PresaleArgs,
    locked_vesting_params: LockedVestingArgs,
) {
    let mut errs = vec![];

    let mut invalid_locked_vesting_params = locked_vesting_params.clone();
    invalid_locked_vesting_params.lock_duration = MAXIMUM_LOCK_AND_VEST_DURATION / 2;
    invalid_locked_vesting_params.vest_duration = MAXIMUM_LOCK_AND_VEST_DURATION / 2 + 1;

    let err_0 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params,
            locked_vesting_params: Some(invalid_locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    errs.push(err_0);

    let expected_err = presale::errors::PresaleError::InvalidLockVestingInfo;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    for err in errs {
        assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
    }
}

fn assert_err_invalid_presale_params(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
    mint: Pubkey,
    quote_mint: Pubkey,
    tokenomic: TokenomicArgs,
    presale_params: PresaleArgs,
    locked_vesting_params: LockedVestingArgs,
) {
    let mut errs = vec![];

    let mut invalid_presale_params = presale_params.clone();
    invalid_presale_params.presale_maximum_cap = 0;
    invalid_presale_params.presale_minimum_cap = 0;

    let err_0 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_0);

    invalid_presale_params = presale_params.clone();
    invalid_presale_params.presale_maximum_cap = 100;
    invalid_presale_params.presale_minimum_cap = 200;

    let err_1 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_1);

    invalid_presale_params = presale_params.clone();
    let clock = lite_svm.get_sysvar::<Clock>();
    invalid_presale_params.presale_start_time = clock.unix_timestamp as u64;
    invalid_presale_params.presale_end_time =
        invalid_presale_params.presale_start_time + MINIMUM_PRESALE_DURATION - 1;

    let err_2 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_2);

    invalid_presale_params = presale_params.clone();
    invalid_presale_params.presale_start_time = clock.unix_timestamp as u64 + 100;
    invalid_presale_params.presale_end_time = clock.unix_timestamp as u64 + 50;

    let err_3 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_3);

    invalid_presale_params = presale_params.clone();
    invalid_presale_params.presale_start_time = clock.unix_timestamp as u64 + 100;
    invalid_presale_params.presale_end_time =
        invalid_presale_params.presale_start_time + MAXIMUM_PRESALE_DURATION + 1;

    let err_4 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_4);

    invalid_presale_params = presale_params.clone();
    invalid_presale_params.presale_start_time =
        clock.unix_timestamp as u64 + MAXIMUM_DURATION_UNTIL_PRESALE + 1;
    invalid_presale_params.presale_end_time = invalid_presale_params.presale_start_time + 1000;

    let err_5 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_5);

    invalid_presale_params = presale_params.clone();
    invalid_presale_params.presale_maximum_cap = 100;
    invalid_presale_params.presale_minimum_cap = 0;

    let err_6 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_6);

    invalid_presale_params = presale_params.clone();
    invalid_presale_params.buyer_maximum_deposit_cap = 0;
    invalid_presale_params.buyer_minimum_deposit_cap = 0;

    let err_7 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_7);

    invalid_presale_params = presale_params.clone();
    invalid_presale_params.buyer_maximum_deposit_cap = 100;
    invalid_presale_params.buyer_minimum_deposit_cap = 200;

    let err_8 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    errs.push(err_8);

    invalid_presale_params = presale_params.clone();
    invalid_presale_params.buyer_maximum_deposit_cap = 100;
    invalid_presale_params.buyer_minimum_deposit_cap = 50;
    invalid_presale_params.presale_maximum_cap = 90;
    invalid_presale_params.presale_minimum_cap = 50;

    let err_9 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_9);

    let expected_err = presale::errors::PresaleError::InvalidPresaleInfo;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    for err in errs {
        assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
    }
}

fn assert_err_buyer_max_cap_cannot_purchase_even_a_single_token(setup_context: &mut SetupContext) {
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let lite_svm = &mut setup_context.lite_svm;
    let user = Rc::clone(&setup_context.user);

    let user_pubkey = user.pubkey();

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let price = 0.01;

    handle_initialize_fixed_token_price_presale_params(
        lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: mint,
            quote_mint,
            q_price: calculate_q_price_from_ui_price(
                price,
                DEFAULT_BASE_TOKEN_DECIMALS,
                DEFAULT_QUOTE_TOKEN_DECIMALS,
            ),
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let presale_pool_supply = 1_000_000; // 1 million

    let tokenomic = TokenomicArgs {
        presale_pool_supply: presale_pool_supply * 10u64.pow(6),
        ..Default::default()
    };

    let lamport_price = price
        * 10f64
            .powi(i32::from(DEFAULT_QUOTE_TOKEN_DECIMALS) - i32::from(DEFAULT_BASE_TOKEN_DECIMALS));

    println!("Lamport price: {}", lamport_price);

    let buyer_maximum_deposit_cap = (lamport_price - 1.0f64) as u64;
    println!("Buyer max cap: {}", buyer_maximum_deposit_cap);

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        buyer_maximum_deposit_cap,
        buyer_minimum_deposit_cap: 0,
        whitelist_mode: WhitelistMode::Permissionless.into(),
        ..Default::default()
    };

    let err = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params,
            locked_vesting_params: None,
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![AccountMeta {
                pubkey: derive_fixed_price_presale_args(
                    &mint,
                    &quote_mint,
                    &user_pubkey,
                    &presale::ID,
                ),
                is_signer: false,
                is_writable: false,
            }],
        },
    );

    let expected_err = presale::errors::PresaleError::ZeroTokenAmount;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;

    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_initialize_fixed_token_price_presale_vault_with_invalid_configuration() {
    let mut setup_context = SetupContext::initialize();
    assert_err_buyer_max_cap_cannot_purchase_even_a_single_token(&mut setup_context);
}

#[test]
fn test_initialize_fixed_token_price_presale_vault_missing_fixed_price_extra_args_remaining_accounts(
) {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;

    let tokenomic = create_tokenomic_args(DEFAULT_BASE_TOKEN_DECIMALS);
    let presale_params = create_presale_args(&lite_svm);

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let user_pubkey = user.pubkey();

    let err = handle_initialize_presale_err(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params,
            locked_vesting_params: None,
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![], // Missing fixed price presale args account
        },
    );

    let expected_err = presale::errors::PresaleError::MissingPresaleExtraParams;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_initialize_presale_vault_with_invalid_parameters() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let tokenomic = create_tokenomic_args(DEFAULT_BASE_TOKEN_DECIMALS);
    let presale_params = create_presale_args(&lite_svm);
    let locked_vesting_params = LockedVestingArgs {
        lock_duration: 3600,
        vest_duration: 3600 * 2,
        ..Default::default()
    };

    assert_err_invalid_tokenomic(
        &mut lite_svm,
        Rc::clone(&user),
        mint,
        quote_mint,
        tokenomic,
        presale_params,
        locked_vesting_params,
    );

    assert_err_invalid_presale_params(
        &mut lite_svm,
        Rc::clone(&user),
        mint,
        quote_mint,
        tokenomic,
        presale_params,
        locked_vesting_params,
    );

    assert_err_invalid_locked_vesting_param(
        &mut lite_svm,
        Rc::clone(&user),
        mint,
        quote_mint,
        tokenomic,
        presale_params,
        locked_vesting_params,
    )
}

#[test]
fn test_initialize_presale_vault_with_dynamic_price_fcfs() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let user_pubkey = user.pubkey();

    let tokenomic = TokenomicArgs {
        presale_pool_supply: 1_000_000 * 10u64.pow(6), // 1 million
        ..Default::default()
    };

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::Fcfs.into(),
        buyer_maximum_deposit_cap: LAMPORTS_PER_SOL,
        buyer_minimum_deposit_cap: 1_000_000, // 0.0001 SOL
        whitelist_mode: WhitelistMode::PermissionWithAuthority.into(),
        ..Default::default()
    };

    let lock_vesting_params = LockedVestingArgs {
        lock_duration: 3600,
        vest_duration: 3600 * 2,
        ..Default::default()
    };

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params,
            locked_vesting_params: Some(lock_vesting_params),
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&derive_presale(
            &mint,
            &quote_mint,
            &user_pubkey,
            &presale::ID,
        ))
        .unwrap();

    assert_eq!(presale_state.presale_mode, PresaleMode::Fcfs as u8);
    assert_eq!(presale_state.whitelist_mode, presale_params.whitelist_mode);
    assert_eq!(presale_state.fixed_price_presale_q_price, 0);
    assert_eq!(presale_state.fixed_price_presale_unsold_token_action, 0);
}

#[test]
fn test_initialize_presale_vault_with_dynamic_price_prorata() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let user_pubkey = user.pubkey();

    let tokenomic = TokenomicArgs {
        presale_pool_supply: 1_000_000 * 10u64.pow(6), // 1 million
        ..Default::default()
    };

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::Prorata.into(),
        buyer_maximum_deposit_cap: LAMPORTS_PER_SOL,
        buyer_minimum_deposit_cap: 1_000_000, // 0.0001 SOL
        whitelist_mode: WhitelistMode::PermissionWithMerkleProof.into(),
        ..Default::default()
    };

    let lock_vesting_params = LockedVestingArgs {
        lock_duration: 3600,
        vest_duration: 3600 * 2,
        ..Default::default()
    };

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params,
            locked_vesting_params: Some(lock_vesting_params),
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&derive_presale(
            &mint,
            &quote_mint,
            &user_pubkey,
            &presale::ID,
        ))
        .unwrap();

    assert_eq!(presale_state.presale_mode, PresaleMode::Prorata as u8);
    assert_eq!(presale_state.whitelist_mode, presale_params.whitelist_mode);
    assert_eq!(presale_state.fixed_price_presale_q_price, 0);
    assert_eq!(presale_state.fixed_price_presale_unsold_token_action, 0);
}

#[test]
fn test_initialize_presale_vault_with_fixed_token_price() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let user_pubkey = user.pubkey();

    let q_price = calculate_q_price_from_ui_price(
        0.01,
        DEFAULT_BASE_TOKEN_DECIMALS,
        DEFAULT_QUOTE_TOKEN_DECIMALS,
    );

    let unsold_token_action = UnsoldTokenAction::Refund;

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: mint,
            quote_mint,
            q_price,
            unsold_token_action,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let tokenomic = TokenomicArgs {
        presale_pool_supply: 1_000_000 * 10u64.pow(6), // 1 million
        ..Default::default()
    };

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        buyer_maximum_deposit_cap: LAMPORTS_PER_SOL,
        buyer_minimum_deposit_cap: 1_000_000, // 0.0001 SOL
        whitelist_mode: WhitelistMode::Permissionless.into(),
        ..Default::default()
    };

    let lock_vesting_params = LockedVestingArgs {
        lock_duration: 3600,
        vest_duration: 3600 * 2,
        ..Default::default()
    };

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            tokenomic,
            presale_params,
            locked_vesting_params: Some(lock_vesting_params),
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![AccountMeta {
                pubkey: derive_fixed_price_presale_args(
                    &mint,
                    &quote_mint,
                    &user_pubkey,
                    &presale::ID,
                ),
                is_signer: false,
                is_writable: false,
            }],
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&derive_presale(
            &mint,
            &quote_mint,
            &user_pubkey,
            &presale::ID,
        ))
        .unwrap();

    assert_eq!(presale_state.base_mint, mint);
    assert_eq!(presale_state.quote_mint, quote_mint);
    assert_eq!(presale_state.presale_mode, PresaleMode::FixedPrice as u8);
    assert_eq!(
        presale_state.presale_start_time,
        presale_params.presale_start_time
    );
    assert_eq!(
        presale_state.presale_end_time,
        presale_params.presale_end_time
    );
    assert_eq!(
        presale_state.presale_maximum_cap,
        presale_params.presale_maximum_cap
    );
    assert_eq!(
        presale_state.presale_minimum_cap,
        presale_params.presale_minimum_cap
    );
    assert_eq!(
        presale_state.buyer_maximum_deposit_cap,
        presale_params.buyer_maximum_deposit_cap
    );
    assert_eq!(
        presale_state.buyer_minimum_deposit_cap,
        presale_params.buyer_minimum_deposit_cap
    );
    assert_eq!(presale_state.whitelist_mode, presale_params.whitelist_mode);
    assert_eq!(presale_state.owner, user_pubkey);
    assert_eq!(presale_state.base, user_pubkey);
    assert!(presale_state.created_at > 0);
    assert_eq!(presale_state.has_creator_withdrawn, 0);
    assert_eq!(
        presale_state.vest_duration,
        lock_vesting_params.vest_duration
    );
    assert_eq!(
        presale_state.lock_duration,
        lock_vesting_params.lock_duration
    );

    assert_eq!(
        presale_state.lock_start_time,
        presale_state.presale_end_time + 1
    );
    assert_eq!(
        presale_state.lock_end_time,
        presale_state.lock_start_time + lock_vesting_params.lock_duration
    );
    assert_eq!(
        presale_state.vesting_start_time,
        presale_state.lock_end_time + 1
    );
    assert_eq!(
        presale_state.vesting_end_time,
        presale_state.vesting_start_time + lock_vesting_params.vest_duration
    );
    assert_eq!(presale_state.fixed_price_presale_q_price, q_price);
    assert_eq!(
        presale_state.fixed_price_presale_unsold_token_action,
        unsold_token_action as u8
    );

    let base_vault_token_account: TokenAccount = lite_svm
        .get_deserialized_account(&presale_state.base_token_vault)
        .unwrap();

    assert_eq!(
        base_vault_token_account.amount,
        tokenomic.presale_pool_supply
    );
}
