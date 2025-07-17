pub mod helpers;

use anchor_lang::{
    error::ERROR_CODE_OFFSET,
    prelude::{AccountMeta, Clock},
};
use helpers::*;
use litesvm::LiteSVM;
use std::rc::Rc;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use presale::{PresaleArgs, PresaleMode, TokenomicArgs, UnsoldTokenAction, WhitelistMode};

fn assert_err_buyer_max_cap_cannot_purchase_even_a_single_token(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
) {
    let user_pubkey = user.pubkey();

    let base_mint = Rc::new(Keypair::new());

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let token_info = create_token_info();

    let price = 0.01;
    handle_initialize_fixed_token_price_presale_params(
        lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint.pubkey(),
            quote_mint,
            q_price: calculate_q_price_from_ui_price(price, token_info.decimals, 9),
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
        },
    );

    let presale_pool_supply = 1_000_000;

    let tokenomic = TokenomicArgs {
        presale_pool_supply: presale_pool_supply * 10u64.pow(token_info.decimals.into()),
        creator_supply: 0,
    };

    let token_per_sol = 1.0 / price;
    let base_lamport_per_sol_lamport =
        token_per_sol * 10.0f64.powi(i32::from(token_info.decimals) - 9);

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
        presale_maximum_cap: 1 * LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        buyer_maximum_deposit_cap,
        buyer_minimum_deposit_cap: 0,
        max_deposit_fee: 0,
        deposit_fee_bps: 0,
        whitelist_mode: WhitelistMode::Permissionless.into(),
    };

    let err = handle_initialize_presale_err(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint: Rc::clone(&base_mint),
            quote_mint,
            token_info,
            tokenomic,
            presale_params,
            locked_vesting_params: None,
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![AccountMeta {
                pubkey: derive_fixed_price_presale_args(
                    &base_mint.pubkey(),
                    &quote_mint,
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
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();
    assert_err_buyer_max_cap_cannot_purchase_even_a_single_token(&mut lite_svm, Rc::clone(&user));
}

#[test]
fn test_initialize_presale_vault_with_fixed_token_price() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();
    let user_pubkey = user.pubkey();

    let base_mint = Rc::new(Keypair::new());

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let token_info = create_token_info();

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint.pubkey(),
            quote_mint,
            q_price: calculate_q_price_from_ui_price(0.01, token_info.decimals, 9),
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
        },
    );

    let tokenomic = TokenomicArgs {
        presale_pool_supply: 1_000_000 * 10u64.pow(token_info.decimals.into()), // 1 million
        creator_supply: 0,
    };

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: 1 * LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        buyer_maximum_deposit_cap: u64::MAX,
        buyer_minimum_deposit_cap: 1_000_000, // 0.0001 SOL
        max_deposit_fee: 0,
        deposit_fee_bps: 0,
        whitelist_mode: WhitelistMode::Permissionless.into(),
    };

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: Rc::clone(&base_mint),
            quote_mint,
            token_info,
            tokenomic,
            presale_params,
            locked_vesting_params: None,
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![AccountMeta {
                pubkey: derive_fixed_price_presale_args(
                    &base_mint.pubkey(),
                    &quote_mint,
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
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();
    let user_pubkey = user.pubkey();

    let base_mint = Rc::new(Keypair::new());
    let quote_mint = Rc::new(Keypair::new());

    let base_mint_pubkey = base_mint.pubkey();
    let quote_mint_pubkey = quote_mint.pubkey();

    create_token_2022(CreateToken2022Args {
        lite_svm: &mut lite_svm,
        mint: Rc::clone(&quote_mint),
        mint_authority: Rc::clone(&user),
        payer: Rc::clone(&user),
        decimals: 9,
    });

    let token_info = create_token_info();

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            q_price: calculate_q_price_from_ui_price(0.01, token_info.decimals, 9),
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
        },
    );

    let tokenomic = TokenomicArgs {
        presale_pool_supply: 1_000_000 * 10u64.pow(token_info.decimals.into()), // 1 million
        creator_supply: 0,
    };

    let clock: Clock = lite_svm.get_sysvar();

    let presale_params = PresaleArgs {
        presale_start_time: clock.unix_timestamp as u64,
        presale_end_time: clock.unix_timestamp as u64 + 120,
        presale_maximum_cap: 1 * LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        buyer_maximum_deposit_cap: u64::MAX,
        buyer_minimum_deposit_cap: 1_000_000, // 0.0001 SOL
        max_deposit_fee: 0,
        deposit_fee_bps: 0,
        whitelist_mode: WhitelistMode::Permissionless.into(),
    };

    handle_initialize_presale_token_2022(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: Rc::clone(&base_mint),
            quote_mint: quote_mint_pubkey,
            token_info,
            tokenomic,
            presale_params,
            locked_vesting_params: None,
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![AccountMeta {
                pubkey: derive_fixed_price_presale_args(
                    &base_mint_pubkey,
                    &quote_mint_pubkey,
                    &presale::ID,
                ),
                is_signer: false,
                is_writable: false,
            }],
        },
    );
}
