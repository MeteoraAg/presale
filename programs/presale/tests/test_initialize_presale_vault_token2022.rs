use std::rc::Rc;

use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::prelude::AccountMeta;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_interface::TokenAccount,
};
use presale::{Presale, UnsoldTokenAction};

use crate::helpers::{
    calculate_q_price_from_ui_price, create_default_presale_registries, create_presale_args,
    derive_fixed_price_presale_args, derive_presale,
    handle_initialize_fixed_token_price_presale_params, handle_initialize_presale,
    HandleInitializeFixedTokenPricePresaleParamsArgs, HandleInitializePresaleArgs, LiteSVMExt,
    SetupContext, DEFAULT_BASE_TOKEN_DECIMALS, DEFAULT_QUOTE_TOKEN_DECIMALS,
    PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
};

pub mod helpers;

#[test]
fn test_initialize_presale_vault_token_2022() {
    let mut setup_context = SetupContext::initialize();
    let base_mint_pubkey = setup_context.setup_token_2022_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let quote_mint_pubkey = anchor_spl::token::spl_token::native_mint::ID;

    let SetupContext { mut lite_svm, user } = setup_context;
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
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            q_price,
            unsold_token_action,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let presale_registries = create_default_presale_registries(
        DEFAULT_BASE_TOKEN_DECIMALS,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let presale_params = create_presale_args(&lite_svm);

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            presale_registries,
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

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&derive_presale(
            &base_mint_pubkey,
            &quote_mint_pubkey,
            &user_pubkey,
            &presale::ID,
        ))
        .unwrap();

    let base_token_vault: TokenAccount = lite_svm
        .get_deserialized_account(&presale_state.base_token_vault)
        .unwrap();

    let presale_pool_supply = presale_state
        .presale_registries
        .iter()
        .map(|r| r.presale_supply)
        .sum::<u64>();

    assert_eq!(base_token_vault.amount, presale_pool_supply);
}

#[test]
fn test_initialize_presale_vault_token_2022_with_transfer_fee() {
    let mut setup_context = SetupContext::initialize();
    let base_mint_pubkey = setup_context.setup_token_2022_mint_with_transfer_fee(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let quote_mint_pubkey = anchor_spl::token::spl_token::native_mint::ID;

    let SetupContext { mut lite_svm, user } = setup_context;
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
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            q_price,
            unsold_token_action,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let presale_registries = create_default_presale_registries(
        DEFAULT_BASE_TOKEN_DECIMALS,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let presale_params = create_presale_args(&lite_svm);

    let user_base_ata = get_associated_token_address_with_program_id(
        &user_pubkey,
        &base_mint_pubkey,
        &anchor_spl::token_2022::spl_token_2022::ID,
    );

    let before_user_base_token: TokenAccount =
        lite_svm.get_deserialized_account(&user_base_ata).unwrap();

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            presale_registries,
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

    let after_user_base_token: TokenAccount =
        lite_svm.get_deserialized_account(&user_base_ata).unwrap();

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&derive_presale(
            &base_mint_pubkey,
            &quote_mint_pubkey,
            &user_pubkey,
            &presale::ID,
        ))
        .unwrap();

    let deducted_amount = before_user_base_token.amount - after_user_base_token.amount;
    // Transfer fee
    let presale_pool_supply = presale_state
        .presale_registries
        .iter()
        .map(|r| r.presale_supply)
        .sum::<u64>();

    assert!(deducted_amount > presale_pool_supply);

    let base_token_vault: TokenAccount = lite_svm
        .get_deserialized_account(&presale_state.base_token_vault)
        .unwrap();

    assert_eq!(base_token_vault.amount, presale_pool_supply);
}

#[test]
fn test_initialize_presale_vault_token_2022_with_transfer_hook() {
    let mut setup_context = SetupContext::initialize();
    let base_mint_pubkey = setup_context.setup_token_2022_mint_with_transfer_hook(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let quote_mint_pubkey = anchor_spl::token::spl_token::native_mint::ID;

    let SetupContext { mut lite_svm, user } = setup_context;
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
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            q_price,
            unsold_token_action,
            owner: user_pubkey,
            payer: Rc::clone(&user),
            base: user_pubkey,
        },
    );

    let presale_registries = create_default_presale_registries(
        DEFAULT_BASE_TOKEN_DECIMALS,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );
    let presale_params = create_presale_args(&lite_svm);

    handle_initialize_presale(
        &mut lite_svm,
        HandleInitializePresaleArgs {
            base_mint: base_mint_pubkey,
            quote_mint: quote_mint_pubkey,
            presale_registries,
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

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&derive_presale(
            &base_mint_pubkey,
            &quote_mint_pubkey,
            &user_pubkey,
            &presale::ID,
        ))
        .unwrap();

    let base_token_vault: TokenAccount = lite_svm
        .get_deserialized_account(&presale_state.base_token_vault)
        .unwrap();

    let presale_pool_supply = presale_state
        .presale_registries
        .iter()
        .map(|r| r.presale_supply)
        .sum::<u64>();

    assert_eq!(base_token_vault.amount, presale_pool_supply);
}
