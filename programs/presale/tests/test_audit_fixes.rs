use std::rc::Rc;

use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::error::ERROR_CODE_OFFSET;
use presale::{Presale, DEFAULT_PERMISSIONLESS_REGISTRY_INDEX};

use crate::helpers::{
    create_deposit_ix, create_escrow_withdraw_ix, create_escrow_withdraw_remaining_quote_ix,
    derive_escrow, handle_close_escrow_ix,
    handle_create_predefined_permissionless_fixed_price_presale, handle_escrow_deposit,
    process_transaction, warp_time, HandleCloseEscrowArgs, HandleCreatePredefinedPresaleResponse,
    HandleEscrowDepositArgs, HandleEscrowWithdrawArgs, HandleEscrowWithdrawRemainingQuoteArgs,
    LiteSVMExt, SetupContext, DEFAULT_BASE_TOKEN_DECIMALS,
};

pub mod helpers;

#[test]
fn test_presale_progress_manipulation() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

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

    let deposit_ixs = create_deposit_ix(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            max_amount: presale_state.presale_maximum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            owner: Rc::clone(&user),
        },
    );

    let withdraw_amount =
        (presale_state.presale_maximum_cap - presale_state.presale_minimum_cap) + 1;

    let withdraw_ixs = create_escrow_withdraw_ix(
        &lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            amount: withdraw_amount,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let mut instructions = vec![];
    instructions.extend_from_slice(&deposit_ixs);
    instructions.extend_from_slice(&withdraw_ixs);

    let err = process_transaction(&mut lite_svm, &instructions, Some(&user_pubkey), &[&user])
        .unwrap_err();

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForWithdraw;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_uncloseable_escrow_on_failed_presale() {
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
            max_amount: presale_state.presale_minimum_cap - 1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let withdraw_remaining_quote_ixs = create_escrow_withdraw_remaining_quote_ix(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let close_escrow_ixs = handle_close_escrow_ix(HandleCloseEscrowArgs {
        presale: presale_pubkey,
        owner: Rc::clone(&user),
        registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
    });

    let mut instructions = vec![];
    instructions.extend_from_slice(&withdraw_remaining_quote_ixs);
    instructions.extend_from_slice(&close_escrow_ixs);

    process_transaction(&mut lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let escrow = derive_escrow(
        &presale_pubkey,
        &user.pubkey(),
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );

    let escrow_account = lite_svm.get_account(&escrow).unwrap();
    assert_eq!(escrow_account.owner, anchor_lang::system_program::ID);
}
