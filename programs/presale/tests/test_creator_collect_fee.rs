use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer};
use anchor_lang::{error::ERROR_CODE_OFFSET, AccountDeserialize};
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_interface::TokenAccount,
};
use presale::{Presale, DEFAULT_PERMISSIONLESS_REGISTRY_INDEX};

use crate::helpers::*;
use std::rc::Rc;
pub mod helpers;

#[test]
fn test_creator_collect_fee_before_presale_ends() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale_with_deposit_fee(
            &mut lite_svm,
            mint,
            quote_mint,
            Rc::clone(&user),
        );

    let err = handle_creator_collect_fee_err(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForCollectFee;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_non_creator_collect_fee() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();

    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale_with_deposit_fee(
            &mut lite_svm,
            mint,
            quote_mint,
            Rc::clone(&user),
        );

    let err = handle_creator_collect_fee_err(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            // Invalid user
            owner: Rc::clone(&user_1),
        },
    );

    let err_code = anchor_lang::error::ErrorCode::ConstraintHasOne as u16;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_collect_fee_when_claimed() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale_with_deposit_fee(
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
            max_amount: presale_state.presale_maximum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    handle_creator_collect_fee(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert!(presale_state.is_deposit_fee_collected());

    let err = handle_creator_collect_fee_err(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForCollectFee;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_creator_collect_fee_on_zero_deposit_fee_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let user_pubkey = user.pubkey();
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
            max_amount: presale_state.presale_maximum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let owner_quote_token_address = get_associated_token_address_with_program_id(
        &user_pubkey,
        &quote_mint,
        &anchor_spl::token::spl_token::ID,
    );

    let before_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    handle_creator_collect_fee(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let after_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    let before_balance =
        TokenAccount::try_deserialize(&mut before_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    let after_balance =
        TokenAccount::try_deserialize(&mut after_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    assert_eq!(before_balance, after_balance);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert!(presale_state.is_deposit_fee_collected());
}

#[test]
fn test_creator_collect_fee_prorata_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();
    let SetupContext { mut lite_svm, user } = setup_context;

    let user_1_pubkey = user_1.pubkey();
    let user_pubkey = user.pubkey();

    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        10 * LAMPORTS_PER_SOL,
    );

    wrap_sol(&mut lite_svm, Rc::clone(&user_1), 9 * LAMPORTS_PER_SOL);

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale_with_deposit_fee(
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
            max_amount: presale_state.presale_maximum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: presale_state.presale_maximum_cap,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    // Collect all fund raised first, implicitly test escrow refund deposit fee won't drain the reserve
    handle_creator_withdraw_token(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    for user in [&user, &user_1] {
        handle_escrow_withdraw_remaining_quote(
            &mut lite_svm,
            HandleEscrowWithdrawRemainingQuoteArgs {
                presale: presale_pubkey,
                owner: Rc::clone(user),
                registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            },
        );
    }

    let owner_quote_token_address = get_associated_token_address_with_program_id(
        &user_pubkey,
        &quote_mint,
        &anchor_spl::token::spl_token::ID,
    );

    let before_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    handle_creator_collect_fee(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let after_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    let before_balance =
        TokenAccount::try_deserialize(&mut before_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    let after_balance =
        TokenAccount::try_deserialize(&mut after_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    assert!(after_balance > before_balance);
    let collected_fee = after_balance - before_balance;

    // Due to prorata need to refund deposit fee of remaining quote
    assert!(collected_fee < presale_state.total_deposit_fee);

    assert!(presale_state.is_deposit_fee_collected());
}

#[test]
fn test_creator_collect_fee_fixed_price_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();
    let SetupContext { mut lite_svm, user } = setup_context;

    let user_1_pubkey = user_1.pubkey();
    let user_pubkey = user.pubkey();

    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        10 * LAMPORTS_PER_SOL,
    );

    wrap_sol(&mut lite_svm, Rc::clone(&user_1), 9 * LAMPORTS_PER_SOL);

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale_with_deposit_fee(
            &mut lite_svm,
            mint,
            quote_mint,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let deposit_amount_0 = presale_state.presale_maximum_cap / 2;
    let deposit_amount_1 = presale_state.presale_maximum_cap - deposit_amount_0;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount_0,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: deposit_amount_1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    // Collect all fund raised first, implicitly test escrow refund deposit fee won't drain the reserve
    handle_creator_withdraw_token(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let owner_quote_token_address = get_associated_token_address_with_program_id(
        &user_pubkey,
        &quote_mint,
        &anchor_spl::token::spl_token::ID,
    );

    let before_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    handle_creator_collect_fee(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let after_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    let before_balance =
        TokenAccount::try_deserialize(&mut before_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    let after_balance =
        TokenAccount::try_deserialize(&mut after_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    assert!(after_balance > before_balance);
    let collected_fee = after_balance - before_balance;

    assert_eq!(collected_fee, presale_state.total_deposit_fee);

    assert!(presale_state.is_deposit_fee_collected());
}

#[test]
fn test_creator_collect_fee_fcfs_price_presale() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();
    let SetupContext { mut lite_svm, user } = setup_context;

    let user_1_pubkey = user_1.pubkey();
    let user_pubkey = user.pubkey();

    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        10 * LAMPORTS_PER_SOL,
    );

    wrap_sol(&mut lite_svm, Rc::clone(&user_1), 9 * LAMPORTS_PER_SOL);

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fcfs_presale_with_deposit_fees(
            &mut lite_svm,
            mint,
            quote_mint,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let deposit_amount_0 = presale_state.presale_maximum_cap / 2;
    let deposit_amount_1 = presale_state.presale_maximum_cap - deposit_amount_0;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount_0,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: deposit_amount_1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    // Collect all fund raised first, implicitly test escrow refund deposit fee won't drain the reserve
    handle_creator_withdraw_token(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let owner_quote_token_address = get_associated_token_address_with_program_id(
        &user_pubkey,
        &quote_mint,
        &anchor_spl::token::spl_token::ID,
    );

    let before_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    handle_creator_collect_fee(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let after_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    let before_balance =
        TokenAccount::try_deserialize(&mut before_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    let after_balance =
        TokenAccount::try_deserialize(&mut after_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    assert!(after_balance > before_balance);
    let collected_fee = after_balance - before_balance;

    assert_eq!(collected_fee, presale_state.total_deposit_fee);

    assert!(presale_state.is_deposit_fee_collected());
}

#[test]
fn test_creator_collect_fee_token_2022() {
    let mut setup_context = SetupContext::initialize();
    let base_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let quote_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_QUOTE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_QUOTE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();
    let SetupContext { mut lite_svm, user } = setup_context;

    let user_1_pubkey = user_1.pubkey();
    let user_pubkey = user.pubkey();

    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        LAMPORTS_PER_SOL,
    );

    transfer_token(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        quote_mint,
        1_000_000_000 * 10u64.pow(DEFAULT_QUOTE_TOKEN_DECIMALS.into()),
    );

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fcfs_presale_with_deposit_fees(
            &mut lite_svm,
            base_mint,
            quote_mint,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let deposit_amount_0 = presale_state.presale_maximum_cap / 2;
    let deposit_amount_1 = presale_state.presale_maximum_cap - deposit_amount_0;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount_0,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: deposit_amount_1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    // Collect all fund raised first, implicitly test escrow refund deposit fee won't drain the reserve
    handle_creator_withdraw_token(
        &mut lite_svm,
        HandleCreatorWithdrawTokenArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let owner_quote_token_address = get_associated_token_address_with_program_id(
        &user_pubkey,
        &quote_mint,
        &anchor_spl::token_2022::ID,
    );

    let before_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    handle_creator_collect_fee(
        &mut lite_svm,
        HandleCreatorCollectFeeArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let after_owner_quote_account = lite_svm.get_account(&owner_quote_token_address);

    let before_balance =
        TokenAccount::try_deserialize(&mut before_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    let after_balance =
        TokenAccount::try_deserialize(&mut after_owner_quote_account.unwrap().data.as_ref())
            .unwrap()
            .amount;

    assert!(after_balance > before_balance);
    let collected_fee = after_balance - before_balance;

    // Due to transfer fee
    assert!(collected_fee < presale_state.total_deposit_fee);

    assert!(presale_state.is_deposit_fee_collected());
}
