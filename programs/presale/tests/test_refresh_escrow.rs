use std::rc::Rc;

use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::prelude::Clock;
use presale::{Escrow, Presale, DEFAULT_PERMISSIONLESS_REGISTRY_INDEX};

use crate::helpers::{
    derive_escrow, handle_create_predefined_permissionless_fixed_price_presale,
    handle_escrow_deposit, handle_escrow_refresh, warp_time, HandleCreatePredefinedPresaleResponse,
    HandleEscrowDepositArgs, HandleEscrowRefreshArgs, LiteSVMExt, SetupContext,
    DEFAULT_BASE_TOKEN_DECIMALS,
};

pub mod helpers;

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
