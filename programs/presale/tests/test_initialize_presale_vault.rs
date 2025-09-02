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
    LockedVestingArgs, Presale, PresaleArgs, PresaleMode, PresaleRegistryArgs, WhitelistMode,
    MAXIMUM_DURATION_UNTIL_PRESALE, MAXIMUM_LOCK_AND_VEST_DURATION, MAXIMUM_PRESALE_DURATION,
    MAX_PRESALE_REGISTRY_COUNT, MINIMUM_PRESALE_DURATION,
};

fn assert_err_invalid_locked_vesting_param(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
    mint: Pubkey,
    quote_mint: Pubkey,
    presale_registries: Vec<PresaleRegistryArgs>,
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
            presale_registries,
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
    presale_registries: Vec<PresaleRegistryArgs>,
    presale_params: PresaleArgs,
    locked_vesting_params: LockedVestingArgs,
) {
    let mut errs = vec![];

    let valid_presale_registries = presale_registries.clone();

    let mut invalid_presale_params = presale_params.clone();
    invalid_presale_params.presale_maximum_cap = 0;
    invalid_presale_params.presale_minimum_cap = 0;

    let err_0 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries: valid_presale_registries.clone(),
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
            presale_registries: valid_presale_registries.clone(),
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
            presale_registries: valid_presale_registries.clone(),
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
            presale_registries: valid_presale_registries.clone(),
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
            presale_registries: valid_presale_registries.clone(),
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
            presale_registries: valid_presale_registries.clone(),
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
            presale_registries: valid_presale_registries.clone(),
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_6);

    invalid_presale_params = presale_params.clone();

    // Un-depositable
    let mut invalid_presale_registries = vec![];
    let mut invalid_registry = PresaleRegistryArgs::default();
    invalid_registry.presale_supply = 1000;
    invalid_presale_registries.push(invalid_registry);

    let err_7 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries: invalid_presale_registries,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_7);

    invalid_presale_params = presale_params.clone();

    // Minimum > maximum
    let mut invalid_presale_registries = vec![];
    let mut invalid_registry = PresaleRegistryArgs::default();
    invalid_registry.presale_supply = 1000;
    invalid_registry.buyer_maximum_deposit_cap = 100;
    invalid_registry.buyer_minimum_deposit_cap = 200;
    invalid_presale_registries.push(invalid_registry);

    let err_8 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries: invalid_presale_registries,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    errs.push(err_8);

    invalid_presale_params = presale_params.clone();
    // Unreachable buyer maximum deposit cap due to > presale maximum cap
    let mut invalid_presale_registries = vec![];
    let mut invalid_registry = PresaleRegistryArgs::default();
    invalid_registry.presale_supply = 1000;
    invalid_registry.buyer_maximum_deposit_cap = 100;
    invalid_registry.buyer_minimum_deposit_cap = 200;
    invalid_presale_params.presale_maximum_cap = 90;
    invalid_presale_params.presale_minimum_cap = 50;
    invalid_presale_registries.push(invalid_registry);

    let err_9 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries: invalid_presale_registries,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );
    errs.push(err_9);

    // Presale have 0 registries
    let err_10 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries: vec![],
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    errs.push(err_10);

    let expected_err = presale::errors::PresaleError::InvalidPresaleInfo;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    for err in errs.iter() {
        assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
    }

    errs.clear();

    // Presale registry have 0 supply
    let mut invalid_presale_registries = vec![];
    let mut invalid_registry = PresaleRegistryArgs::default();
    invalid_registry.buyer_minimum_deposit_cap = 0;
    invalid_registry.buyer_maximum_deposit_cap = 50;
    invalid_presale_registries.push(invalid_registry);

    let err_0 = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries: invalid_presale_registries,
            presale_params: invalid_presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user.pubkey(),
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    let expected_err = presale::errors::PresaleError::InvalidTokenSupply;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err_0.meta.logs.iter().any(|log| log.contains(&err_str)));
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
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let presale_pool_supply = 1_000_000 * 10u64.pow(6); // 1 million

    let lamport_price = price
        * 10f64
            .powi(i32::from(DEFAULT_QUOTE_TOKEN_DECIMALS) - i32::from(DEFAULT_BASE_TOKEN_DECIMALS));

    println!("Lamport price: {}", lamport_price);

    let buyer_maximum_deposit_cap = (lamport_price - 1.0f64) as u64;
    println!("Buyer max cap: {}", buyer_maximum_deposit_cap);

    let clock: Clock = lite_svm.get_sysvar();

    let mut presale_registries = vec![];
    let mut registry = PresaleRegistryArgs::default();
    registry.presale_supply = presale_pool_supply;
    registry.buyer_maximum_deposit_cap = buyer_maximum_deposit_cap;
    registry.buyer_minimum_deposit_cap = 0;
    presale_registries.push(registry);

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        whitelist_mode: WhitelistMode::Permissionless.into(),
        ..Default::default()
    };

    let err = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries,
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

fn assert_err_presale_not_enough_supply_to_fulfill_presale_max_cap(
    setup_context: &mut SetupContext,
) {
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
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let presale_pool_supply = 1_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()); // 1 million

    let lamport_price = price
        * 10f64
            .powi(i32::from(DEFAULT_QUOTE_TOKEN_DECIMALS) - i32::from(DEFAULT_BASE_TOKEN_DECIMALS));

    println!("Lamport price: {}", lamport_price);

    let presale_maximum_cap = ((presale_pool_supply + 1) as f64 * lamport_price) as u64;
    println!("Presale max cap: {}", presale_maximum_cap);

    let clock: Clock = lite_svm.get_sysvar();

    let mut presale_registries = vec![];
    let mut registry = PresaleRegistryArgs::default();
    registry.presale_supply = presale_pool_supply;
    registry.buyer_maximum_deposit_cap = presale_maximum_cap / 2;
    registry.buyer_minimum_deposit_cap = 0;
    presale_registries.push(registry);

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap,
        presale_minimum_cap: 1,
        presale_mode: PresaleMode::FixedPrice.into(),
        whitelist_mode: WhitelistMode::Permissionless.into(),
        ..Default::default()
    };

    let err = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries,
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

    let expected_err = presale::errors::PresaleError::InvalidTokenPrice;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;

    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_initialize_fixed_token_price_presale_vault_with_invalid_configuration() {
    let mut setup_context = SetupContext::initialize();
    assert_err_buyer_max_cap_cannot_purchase_even_a_single_token(&mut setup_context);
    assert_err_presale_not_enough_supply_to_fulfill_presale_max_cap(&mut setup_context);
}

#[test]
fn test_initialize_permissionless_presale_vault_with_multiple_registries() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;

    let mut presale_registries = [PresaleRegistryArgs::default(); MAX_PRESALE_REGISTRY_COUNT];
    for registry in presale_registries.iter_mut() {
        registry.presale_supply = 1000 * 10u64.pow(6);
        registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
        registry.buyer_minimum_deposit_cap = 0;
    }

    let presale_params = create_presale_args(&lite_svm);
    let locked_vesting_params = None;
    let user_pubkey = user.pubkey();

    let err = handle_initialize_presale_err(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint: anchor_spl::token::spl_token::native_mint::ID,
            presale_params,
            presale_registries: presale_registries.to_vec(),
            locked_vesting_params,
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    let expected_err = presale::errors::PresaleError::MultiplePresaleRegistriesNotAllowed;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_total_presale_supply_from_multiple_registries_overflow() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;

    let mut presale_registries = [PresaleRegistryArgs::default(); MAX_PRESALE_REGISTRY_COUNT];
    for registry in presale_registries.iter_mut() {
        registry.presale_supply = u64::MAX;
        registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
        registry.buyer_minimum_deposit_cap = 0;
    }

    let presale_params = create_presale_args(&lite_svm);
    let locked_vesting_params = None;
    let user_pubkey = user.pubkey();

    let err = handle_initialize_presale_err(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint: anchor_spl::token::spl_token::native_mint::ID,
            presale_params,
            presale_registries: presale_registries.to_vec(),
            locked_vesting_params,
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    );

    let expected_err = presale::errors::PresaleError::InvalidTokenSupply;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
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

    let presale_registries = create_default_presale_registries(
        DEFAULT_BASE_TOKEN_DECIMALS,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );
    let presale_params = create_presale_args(&lite_svm);

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let user_pubkey = user.pubkey();

    let err = handle_initialize_presale_err(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries,
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

    let presale_registries = create_default_presale_registries(
        DEFAULT_BASE_TOKEN_DECIMALS,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let presale_params = create_presale_args(&lite_svm);
    let locked_vesting_params = LockedVestingArgs {
        lock_duration: 3600,
        vest_duration: 3600 * 2,
        ..Default::default()
    };

    assert_err_invalid_presale_params(
        &mut lite_svm,
        Rc::clone(&user),
        mint,
        quote_mint,
        presale_registries.clone(),
        presale_params,
        locked_vesting_params,
    );

    assert_err_invalid_locked_vesting_param(
        &mut lite_svm,
        Rc::clone(&user),
        mint,
        quote_mint,
        presale_registries,
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

    let mut presale_registries = vec![];
    let mut registry = PresaleRegistryArgs::default();
    registry.presale_supply = 1_000_000 * 10u64.pow(6); // 1 million
    registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
    registry.buyer_minimum_deposit_cap = 1_000_000; // 0.0001 SOL
    presale_registries.push(registry);

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::Fcfs.into(),
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
            presale_registries,
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
    assert_eq!(presale_state.unsold_token_action, 0);
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

    let mut presale_registries = vec![];
    let mut registry = PresaleRegistryArgs::default();
    registry.presale_supply = 1_000_000 * 10u64.pow(6); // 1 million
    registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
    registry.buyer_minimum_deposit_cap = 1_000_000; // 0.0001 SOL
    presale_registries.push(registry);

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::Prorata.into(),
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
            presale_registries,
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
    assert_eq!(presale_state.unsold_token_action, 0);
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

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: mint,
            quote_mint,
            q_price,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let mut presale_registries = vec![];
    let mut registry = PresaleRegistryArgs::default();
    registry.presale_supply = 1_000_000 * 10u64.pow(6); // 1 million
    registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
    registry.buyer_minimum_deposit_cap = 1_000_000; // 0.0001 SOL
    presale_registries.push(registry);

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL * 2,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        whitelist_mode: WhitelistMode::Permissionless.into(),
        ..Default::default()
    };

    let lock_vesting_params = LockedVestingArgs {
        immediately_release_bps: 5000,
        lock_duration: 3600,
        vest_duration: 3600 * 2,
        ..Default::default()
    };

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: mint,
            quote_mint,
            presale_registries: presale_registries.clone(),
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

    for (i, presale_registry_param) in presale_registries.iter().enumerate() {
        let registry = presale_state.presale_registries.get(i).unwrap();
        assert_eq!(
            presale_registry_param.buyer_maximum_deposit_cap,
            registry.buyer_maximum_deposit_cap
        );
        assert_eq!(
            presale_registry_param.buyer_minimum_deposit_cap,
            registry.buyer_minimum_deposit_cap
        );
        assert_eq!(
            presale_registry_param.presale_supply,
            registry.presale_supply
        );
    }

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
        presale_state.unsold_token_action,
        presale_params.unsold_token_action
    );
    assert_eq!(
        presale_state.immediate_release_bps,
        lock_vesting_params.immediately_release_bps
    );

    let base_vault_token_account: TokenAccount = lite_svm
        .get_deserialized_account(&presale_state.base_token_vault)
        .unwrap();

    let presale_pool_supply = presale_state
        .presale_registries
        .iter()
        .map(|r| r.presale_supply)
        .sum::<u64>();

    assert_eq!(base_vault_token_account.amount, presale_pool_supply);

    let initialized_registry_count = presale_state
        .presale_registries
        .iter()
        .filter(|r| r.presale_supply > 0)
        .count();

    assert_eq!(
        usize::from(presale_state.total_presale_registry_count),
        initialized_registry_count
    );
}

#[test]
fn test_initialize_presale_vault_with_fixed_token_price_with_multiple_registries() {
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

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: mint,
            quote_mint,
            q_price,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let mut presale_registries = vec![];

    let mut registry = PresaleRegistryArgs::default();
    registry.presale_supply = 1_000_000 * 10u64.pow(6); // 1 million
    registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
    registry.buyer_minimum_deposit_cap = 1_000_000; // 0.0001 SOL
    registry.deposit_fee_bps = 100; // 1%

    presale_registries.push(registry);

    let mut registry = PresaleRegistryArgs::default();
    registry.presale_supply = 1_500_000 * 10u64.pow(6); // 1.5 million
    registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
    registry.buyer_minimum_deposit_cap = 1_000_000; // 0.0001 SOL
    registry.deposit_fee_bps = 200; // 2%

    presale_registries.push(registry);

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: LAMPORTS_PER_SOL * 2,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
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
            presale_registries: presale_registries.clone(),
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

    for (i, presale_registry_params) in presale_registries.iter().enumerate() {
        let registry = presale_state.presale_registries.get(i).unwrap();
        assert_eq!(
            presale_registry_params.buyer_maximum_deposit_cap,
            registry.buyer_maximum_deposit_cap
        );
        assert_eq!(
            presale_registry_params.buyer_minimum_deposit_cap,
            registry.buyer_minimum_deposit_cap
        );
        assert_eq!(
            presale_registry_params.presale_supply,
            registry.presale_supply
        );
        assert_eq!(
            presale_registry_params.deposit_fee_bps,
            registry.deposit_fee_bps
        );
    }

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
        presale_state.unsold_token_action,
        presale_params.unsold_token_action
    );

    let base_vault_token_account: TokenAccount = lite_svm
        .get_deserialized_account(&presale_state.base_token_vault)
        .unwrap();

    let presale_pool_supply = presale_state
        .presale_registries
        .iter()
        .map(|r| r.presale_supply)
        .sum::<u64>();

    assert_eq!(base_vault_token_account.amount, presale_pool_supply);

    let initialized_registry_count = presale_state
        .presale_registries
        .iter()
        .filter(|r| r.presale_supply > 0)
        .count();

    assert_eq!(
        usize::from(presale_state.total_presale_registry_count),
        initialized_registry_count
    );
}
