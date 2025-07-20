pub mod helpers;

use anchor_lang::{
    error::ERROR_CODE_OFFSET,
    prelude::{AccountMeta, Clock},
};
use helpers::*;
use std::rc::Rc;

use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer};
use presale::{PresaleArgs, PresaleMode, TokenomicArgs, UnsoldTokenAction, WhitelistMode};

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
            q_price: calculate_q_price_from_ui_price(price, 6, 9),
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let presale_pool_supply = 1_000_000;

    let tokenomic = TokenomicArgs {
        presale_pool_supply: presale_pool_supply * 10u64.pow(6),
    };

    let token_per_sol = 1.0 / price;
    let base_lamport_per_sol_lamport = token_per_sol * 10.0f64.powi(i32::from(6) - 9);

    // How many base lamport per sol lamport?
    println!(
        "base lamport per sol lamport: {}",
        base_lamport_per_sol_lamport
    );

    // amount * base_lamport_per_sol_lamport = 1
    // amount = 1 / base_lamport_per_sol_lamport

    let buyer_maximum_deposit_cap = 1.0f64 / base_lamport_per_sol_lamport - 1.0;
    println!("buyer maximum deposit cap: {}", buyer_maximum_deposit_cap);

    let buyer_maximum_deposit_cap = buyer_maximum_deposit_cap as u64;

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
fn test_initialize_presale_vault_with_fixed_token_price() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let user_pubkey = user.pubkey();

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: mint,
            quote_mint,
            q_price: calculate_q_price_from_ui_price(0.01, 6, 9),
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let tokenomic = TokenomicArgs {
        presale_pool_supply: 1_000_000 * 10u64.pow(6), // 1 million
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
    };

    handle_initialize_presale(
        &mut lite_svm,
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
}

#[test]
fn test_initialize_presale_vault_token_2022() {
    let mut setup_context = SetupContext::initialize();
    let base_mint_pubkey = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let quote_mint_pubkey = setup_context.setup_token_2022_mint(
        DEFAULT_QUOTE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_QUOTE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            q_price: calculate_q_price_from_ui_price(0.01, 6, 9),
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let tokenomic = TokenomicArgs {
        presale_pool_supply: 1_000_000 * 10u64.pow(6), // 1 million
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
    };

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            tokenomic,
            presale_params,
            locked_vesting_params: None,
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![AccountMeta {
                pubkey: derive_fixed_price_presale_args(
                    &base_mint_pubkey,
                    &quote_mint_pubkey,
                    &user_pubkey,
                    &presale::ID,
                ),
                is_signer: false,
                is_writable: false,
            }],
        },
    );
}
