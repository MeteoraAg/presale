pub mod helpers;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use helpers::*;
use presale::{FixedPricePresaleExtraArgs, UnsoldTokenAction};
use std::rc::Rc;

#[test]
pub fn test_initialize_fixed_token_price_extra_params() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();
    let user_pubkey = user.pubkey();

    let base_mint = Keypair::new();
    let base_mint_pubkey = base_mint.pubkey();
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let ui_price = 0.01; // 0.01 SOL
    let q_price = calculate_q_price_from_ui_price(
        ui_price,
        DEFAULT_BASE_TOKEN_DECIMALS,
        DEFAULT_QUOTE_TOKEN_DECIMALS,
    );

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint_pubkey,
            quote_mint,
            q_price,
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let fixed_price_args_pubkey =
        derive_fixed_price_presale_args(&base_mint_pubkey, &quote_mint, &user_pubkey, &presale::ID);
    let fixed_price_args: FixedPricePresaleExtraArgs = lite_svm
        .get_deserialized_zc_account(&fixed_price_args_pubkey)
        .unwrap();

    let FixedPricePresaleExtraArgs {
        unsold_token_action,
        q_price: q_price_set,
        owner,
        ..
    } = fixed_price_args;

    assert_eq!(unsold_token_action, UnsoldTokenAction::Refund as u8);
    assert_eq!(q_price_set, q_price);
    assert_eq!(owner, user_pubkey);
}

#[test]
pub fn test_close_fixed_token_price_extra_params() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();
    let user_pubkey = user.pubkey();

    let base_mint = Keypair::new();
    let base_mint_pubkey = base_mint.pubkey();
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let ui_price = 0.01; // 0.01 SOL
    let q_price = calculate_q_price_from_ui_price(
        ui_price,
        DEFAULT_BASE_TOKEN_DECIMALS,
        DEFAULT_QUOTE_TOKEN_DECIMALS,
    );

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint_pubkey,
            quote_mint,
            q_price,
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    handle_close_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleCloseFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint_pubkey,
            quote_mint,
            owner: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let fixed_token_price_extra_params_pubkey =
        derive_fixed_price_presale_args(&base_mint_pubkey, &quote_mint, &user_pubkey, &presale::ID);

    let account = lite_svm
        .get_account(&fixed_token_price_extra_params_pubkey)
        .unwrap();

    assert_eq!(account.owner, Pubkey::default());
}

#[test]
pub fn test_non_owner_cannot_close_fixed_token_price_extra_params() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();
    let user_pubkey = user.pubkey();

    let base_mint = Keypair::new();
    let base_mint_pubkey = base_mint.pubkey();
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let ui_price = 0.01; // 0.01 SOL
    let q_price = calculate_q_price_from_ui_price(
        ui_price,
        DEFAULT_BASE_TOKEN_DECIMALS,
        DEFAULT_QUOTE_TOKEN_DECIMALS,
    );

    handle_initialize_fixed_token_price_presale_params(
        &mut lite_svm,
        HandleInitializeFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint_pubkey,
            quote_mint,
            q_price,
            unsold_token_action: UnsoldTokenAction::Refund,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let non_owner = Keypair::new();
    transfer_sol(&mut lite_svm, user, non_owner.pubkey(), 1_000_000);

    let err = handle_close_fixed_token_price_presale_params_err(
        &mut lite_svm,
        HandleCloseFixedTokenPricePresaleParamsArgs {
            base_mint: base_mint_pubkey,
            quote_mint,
            owner: Rc::new(non_owner),
            base: user_pubkey,
        },
    );

    let expected_err = anchor_lang::error::ErrorCode::ConstraintHasOne;
    let err_code = expected_err as u32;

    let err_str = format!("Error Number: {}.", err_code);

    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}
