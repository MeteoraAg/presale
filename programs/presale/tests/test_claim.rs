pub mod helpers;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use anchor_lang::error::ERROR_CODE_OFFSET;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_interface::TokenAccount;
use helpers::*;
use litesvm::LiteSVM;
use presale::{Escrow, Presale};
use std::rc::Rc;

enum Cmp {
    Equal,
    GreaterThan,
    #[allow(dead_code)]
    LessThan,
}

fn claim_and_assert(lite_svm: &mut LiteSVM, user: Rc<Keypair>, presale_pubkey: Pubkey, cmp: Cmp) {
    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let base_token_program = get_program_id_from_token_flag(presale_state.base_token_program_flag);

    let escrow = derive_escrow(&presale_pubkey, &user.pubkey(), &presale::ID);
    let user_token_address = get_associated_token_address_with_program_id(
        &user.pubkey(),
        &presale_state.base_mint,
        &base_token_program,
    );
    let before_escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
    let before_user_token_amount = lite_svm
        .get_deserialized_account::<TokenAccount>(&user_token_address)
        .map(|account| account.amount)
        .unwrap_or(0);

    handle_escrow_claim(
        lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            refresh_escrow: true,
        },
    );

    let after_escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
    let after_user_token_amount = lite_svm
        .get_deserialized_account::<TokenAccount>(&user_token_address)
        .map(|account| account.amount)
        .unwrap_or(0);

    match cmp {
        Cmp::Equal => {
            assert_eq!(
                after_escrow_state.total_claimed_token,
                before_escrow_state.total_claimed_token
            );
            assert_eq!(after_user_token_amount, before_user_token_amount);
        }
        Cmp::GreaterThan => {
            assert!(
                after_escrow_state.total_claimed_token > before_escrow_state.total_claimed_token
            );
            assert!(after_user_token_amount > before_user_token_amount);
        }
        Cmp::LessThan => {
            assert!(
                after_escrow_state.total_claimed_token < before_escrow_state.total_claimed_token
            );
            assert!(after_user_token_amount < before_user_token_amount);
        }
    }
}

#[test]
fn test_claim_non_completed_presale() {
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

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: LAMPORTS_PER_SOL,
        },
    );

    let err = handle_escrow_claim_err(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            refresh_escrow: true,
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForClaim;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_claim_locked_presale() {
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

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: LAMPORTS_PER_SOL,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let err = handle_escrow_claim_err(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            refresh_escrow: true,
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForClaim;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_claim_without_refresh() {
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

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: LAMPORTS_PER_SOL,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.vesting_start_time + 1);

    let err = handle_escrow_claim_err(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            refresh_escrow: false,
        },
    );

    let expected_err = presale::errors::PresaleError::EscrowNotRefreshed;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_claim_token2022() {
    let mut setup_context = SetupContext::initialize();
    let base_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_BASE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let quote_mint = setup_context.setup_token_2022_mint_with_transfer_hook_and_fee(
        DEFAULT_QUOTE_TOKEN_DECIMALS,
        5_000_000_000 * 10u64.pow(DEFAULT_QUOTE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale(
            &mut lite_svm,
            base_mint,
            quote_mint,
            Rc::clone(&user),
        );

    let user_1 = Rc::new(Keypair::new());
    let funding_amount = LAMPORTS_PER_SOL * 3;
    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1.pubkey(),
        LAMPORTS_PER_SOL,
    );
    transfer_token(
        &mut lite_svm,
        Rc::clone(&user),
        user_1.pubkey(),
        quote_mint,
        funding_amount,
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount_0 = presale_state.presale_maximum_cap / 2;
    let amount_1 = presale_state.presale_maximum_cap - amount_0;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount_0,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: amount_1,
        },
    );

    warp_time(
        &mut lite_svm,
        presale_state.vesting_end_time - presale_state.vest_duration / 2,
    );

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        Cmp::GreaterThan,
    );
    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        Cmp::GreaterThan,
    );

    warp_time(&mut lite_svm, presale_state.vesting_end_time);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        Cmp::GreaterThan,
    );
    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        Cmp::GreaterThan,
    );

    claim_and_assert(&mut lite_svm, Rc::clone(&user), presale_pubkey, Cmp::Equal);
}

#[test]
fn test_claim() {
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

    let user_1 = Rc::new(Keypair::new());
    let funding_amount = LAMPORTS_PER_SOL * 3;
    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1.pubkey(),
        funding_amount,
    );
    wrap_sol(
        &mut lite_svm,
        Rc::clone(&user_1),
        funding_amount - LAMPORTS_PER_SOL,
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount_0 = presale_state.presale_maximum_cap / 2;
    let amount_1 = presale_state.presale_maximum_cap - amount_0;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount_0,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: amount_1,
        },
    );

    warp_time(
        &mut lite_svm,
        presale_state.vesting_end_time - presale_state.vest_duration / 2,
    );

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        Cmp::GreaterThan,
    );
    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        Cmp::GreaterThan,
    );

    warp_time(&mut lite_svm, presale_state.vesting_end_time);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        Cmp::GreaterThan,
    );
    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        Cmp::GreaterThan,
    );

    claim_and_assert(&mut lite_svm, Rc::clone(&user), presale_pubkey, Cmp::Equal);
}
