use std::rc::Rc;

use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::prelude::Clock;
use presale::{
    calculate_immediate_release_token, Escrow, FixedPricePresaleHandler, Presale,
    DEFAULT_PERMISSIONLESS_REGISTRY_INDEX, SCALE_OFFSET,
};

use crate::helpers::{
    decode_presale_mode_handler, derive_escrow,
    handle_create_predefined_permissionless_fixed_price_presale,
    handle_create_predefined_permissionless_fixed_price_presale_with_immediate_release,
    handle_escrow_deposit, handle_escrow_refresh, warp_time, warp_to_presale_end,
    HandleCreatePredefinedPresaleResponse, HandleEscrowDepositArgs, HandleEscrowRefreshArgs,
    LiteSVMExt, SetupContext, DEFAULT_BASE_TOKEN_DECIMALS,
};

pub mod helpers;

#[test]
fn test_escrow_refresh_with_immediate_release() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale_with_immediate_release(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
            0,
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: presale_state.presale_minimum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    warp_to_presale_end(&mut lite_svm, &presale_state);

    let escrow = derive_escrow(
        &presale_pubkey,
        &user.pubkey(),
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );

    handle_escrow_refresh(
        &mut lite_svm,
        HandleEscrowRefreshArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let fp_handler = decode_presale_mode_handler::<FixedPricePresaleHandler>(&presale_state);

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    let presale_registry = presale_state
        .get_presale_registry(DEFAULT_PERMISSIONLESS_REGISTRY_INDEX.into())
        .unwrap();

    let registry_sold_token: u64 = ((u128::from(presale_registry.total_deposit)
        << u128::from(SCALE_OFFSET))
        / fp_handler.q_price)
        .try_into()
        .unwrap();

    let token_release =
        calculate_immediate_release_token(registry_sold_token, presale_state.immediate_release_bps)
            .unwrap();

    assert_eq!(
        escrow_state.pending_claim_token,
        token_release.immediate_released_amount
    );

    let current_timestamp = lite_svm.get_sysvar::<Clock>().unix_timestamp as u64;
    assert_eq!(escrow_state.last_refreshed_at, current_timestamp);

    let before_pending_claim_token = escrow_state.pending_claim_token;
    warp_time(&mut lite_svm, presale_state.vesting_start_time);

    handle_escrow_refresh(
        &mut lite_svm,
        HandleEscrowRefreshArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    let after_pending_claim_token = escrow_state.pending_claim_token;
    assert_eq!(after_pending_claim_token, before_pending_claim_token);

    warp_time(
        &mut lite_svm,
        presale_state.vesting_start_time + presale_state.vest_duration / 2,
    );

    handle_escrow_refresh(
        &mut lite_svm,
        HandleEscrowRefreshArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    let after_pending_claim_token = escrow_state.pending_claim_token;
    assert!(after_pending_claim_token > before_pending_claim_token);
}

#[test]
fn test_escrow_refresh() {
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
            max_amount: presale_state.presale_minimum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    warp_time(&mut lite_svm, presale_state.vesting_start_time + 1);

    let escrow = derive_escrow(
        &presale_pubkey,
        &user.pubkey(),
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );

    handle_escrow_refresh(
        &mut lite_svm,
        HandleEscrowRefreshArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert!(escrow_state.pending_claim_token > 0);
    let current_timestamp = lite_svm.get_sysvar::<Clock>().unix_timestamp as u64;
    assert_eq!(escrow_state.last_refreshed_at, current_timestamp);
}
