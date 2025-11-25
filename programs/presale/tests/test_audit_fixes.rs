use std::rc::Rc;

use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::program_pack::Pack;
use anchor_client::solana_sdk::signer::Signer;
use anchor_lang::error::ERROR_CODE_OFFSET;
use anchor_lang::prelude::Clock;
use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_2022::spl_token_2022::state::Account;
use presale::{
    Escrow, Presale, PresaleProgress, DEFAULT_PERMISSIONLESS_REGISTRY_INDEX, SCALE_MULTIPLIER,
};

use crate::helpers::{
    build_merkle_tree, calculate_q_price_from_ui_price,
    create_default_fixed_price_presale_args_wrapper, create_deposit_ix, create_escrow_withdraw_ix,
    create_escrow_withdraw_remaining_quote_ix, derive_escrow, handle_close_escrow_ix,
    handle_create_merkle_root_config, handle_create_permissioned_escrow_with_merkle_proof,
    handle_create_predefined_permissionless_fixed_price_presale,
    handle_create_predefined_permissionless_prorata_presale_with_no_vest_nor_lock,
    handle_escrow_claim, handle_escrow_deposit, handle_escrow_refresh, process_transaction,
    warp_time, CreateDefaultFixedPricePresaleArgsWrapper, HandleCloseEscrowArgs,
    HandleCreateMerkleRootConfigArgs, HandleCreatePermissionedEscrowWithMerkleProofArgs,
    HandleCreatePredefinedPresaleResponse, HandleEscrowClaimArgs, HandleEscrowDepositArgs,
    HandleEscrowRefreshArgs, HandleEscrowWithdrawArgs, HandleEscrowWithdrawRemainingQuoteArgs,
    LiteSVMExt, SetupContext, WhitelistWallet, DEFAULT_BASE_TOKEN_DECIMALS,
};
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signer::keypair::Keypair;
use litesvm::LiteSVM;
use presale::WhitelistMode;

use crate::helpers::{
    derive_presale, handle_escrow_deposit_err, handle_escrow_withdraw, handle_escrow_withdraw_err,
};

pub mod helpers;

#[test]
fn test_fixed_price_presale_registry_consume_over_supply() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let CreateDefaultFixedPricePresaleArgsWrapper {
        mut presale_params_wrapper,
        fixed_point_params_wrapper,
    } = create_default_fixed_price_presale_args_wrapper(
        mint,
        quote_mint,
        &lite_svm,
        WhitelistMode::PermissionWithMerkleProof,
        Rc::clone(&user),
        user_pubkey,
    );

    let presale_pubkey = presale_params_wrapper.accounts.presale;

    let q_price = fixed_point_params_wrapper.args.params.q_price;
    let presale_supply = 1_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into());
    let half_presale_supply = presale_supply / 2;

    presale_params_wrapper
        .args
        .params
        .presale_params
        .presale_minimum_cap = q_price.div_ceil(SCALE_MULTIPLIER).try_into().unwrap();

    let presale_maximum_cap: u64 = ((q_price * u128::from(presale_supply)) >> 64)
        .try_into()
        .unwrap();

    // Round down
    presale_params_wrapper
        .args
        .params
        .presale_params
        .presale_maximum_cap = presale_maximum_cap;

    {
        let registry_0 = presale_params_wrapper
            .args
            .params
            .presale_registries
            .get_mut(0)
            .unwrap();

        registry_0.presale_supply = half_presale_supply;
        registry_0.buyer_minimum_deposit_cap = presale_params_wrapper
            .args
            .params
            .presale_params
            .presale_minimum_cap;
        registry_0.buyer_maximum_deposit_cap = presale_maximum_cap;
    }
    {
        let registry_0 = presale_params_wrapper
            .args
            .params
            .presale_registries
            .first()
            .unwrap();

        presale_params_wrapper
            .args
            .params
            .presale_registries
            .push(*registry_0);
    }

    let init_fp_ix = Instruction {
        program_id: presale::ID,
        accounts: fixed_point_params_wrapper.accounts.to_account_metas(None),
        data: fixed_point_params_wrapper.args.data(),
    };

    let init_presale_ix = presale_params_wrapper.to_instructions();

    let mut instructions = vec![init_fp_ix];
    instructions.extend_from_slice(&init_presale_ix);

    process_transaction(&mut lite_svm, &instructions, Some(&user_pubkey), &[&user]).unwrap();

    let whitelist_wallets = vec![WhitelistWallet {
        address: user_pubkey,
        max_deposit_cap: presale_maximum_cap,
        registry_index: 0,
    }];

    let merkle_tree = build_merkle_tree(whitelist_wallets, 0);
    let tree_node = merkle_tree.get_node(&user_pubkey);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    let merkle_root_config =
        merkle_tree.get_merkle_root_config_pubkey(presale_pubkey, &presale::ID);

    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_root_config,
            registry_index: 0,
            max_deposit_cap: tree_node.deposit_cap,
            proof: tree_node.proof.unwrap(),
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: tree_node.deposit_cap,
            registry_index: 0,
        },
    );

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.vesting_end_time + 1);

    handle_escrow_refresh(
        &mut lite_svm,
        HandleEscrowRefreshArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: 0,
        },
    );

    let escrow = derive_escrow(&presale_pubkey, &user_pubkey, 0, &presale::ID);
    let escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    let escrow_presale_registry = presale_state
        .get_presale_registry(escrow_state.registry_index.into())
        .unwrap();
    println!(
        "pending_claim_token: {}, presale_supply: {}",
        escrow_state.pending_claim_token, escrow_presale_registry.presale_supply
    );
    assert!(escrow_state.pending_claim_token <= escrow_presale_registry.presale_supply);

    let base_mint_account = lite_svm.get_account(&presale_state.base_mint).unwrap();

    let user_token_account = get_associated_token_address_with_program_id(
        &user_pubkey,
        &presale_state.base_mint,
        &base_mint_account.owner,
    );

    let before_user_token_account = lite_svm.get_account(&user_token_account).unwrap();
    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            refresh_escrow: true,
            registry_index: escrow_state.registry_index,
        },
    );
    let after_user_token_account = lite_svm.get_account(&user_token_account).unwrap();

    let before_amount = Account::unpack(&before_user_token_account.data)
        .unwrap()
        .amount;
    let after_amount = Account::unpack(&after_user_token_account.data)
        .unwrap()
        .amount;

    let claimed_amount = after_amount - before_amount;
    assert_eq!(claimed_amount, escrow_state.pending_claim_token);
}

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

#[test]
fn test_zero_vest_duration_dos_escrow_claim() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let user_1 = setup_context.create_user();

    let SetupContext { mut lite_svm, user } = setup_context;
    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_prorata_presale_with_no_vest_nor_lock(
            &mut lite_svm,
            mint,
            quote,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let capital_required = presale_state.presale_maximum_cap - presale_state.presale_minimum_cap;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: capital_required / 2,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: capital_required / 2,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    let escrow_0_address = derive_escrow(
        &presale_pubkey,
        &user.pubkey(),
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );

    let escrow_1_address = derive_escrow(
        &presale_pubkey,
        &user_1.pubkey(),
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        &presale::ID,
    );

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            refresh_escrow: true,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            refresh_escrow: true,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let escrow_0_state: Escrow = lite_svm
        .get_deserialized_zc_account(&escrow_0_address)
        .unwrap();

    let escrow_1_state: Escrow = lite_svm
        .get_deserialized_zc_account(&escrow_1_address)
        .unwrap();

    let total_claimed_token =
        escrow_0_state.total_claimed_token + escrow_1_state.total_claimed_token;

    let presale_registry_0 = presale_state
        .get_presale_registry(DEFAULT_PERMISSIONLESS_REGISTRY_INDEX.into())
        .unwrap();

    assert_eq!(presale_registry_0.presale_supply, total_claimed_token);
}

// https://github.com/sherlock-audit/2025-11-meteora-nov-10th/pull/26
#[test]
fn test_fixed_price_presale_completion_unreachable() {
    let mut setup_context = SetupContext::initialize();
    let base_mint = setup_context.setup_mint(9, 1_000_000_000 * 10u64.pow(9));

    let SetupContext { mut lite_svm, user } = setup_context;

    let user_pubkey = user.pubkey();
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let whitelist_mode = WhitelistMode::Permissionless;

    let mut wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        &lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user_pubkey,
    );

    let presale_pubkey = wrapper.presale_params_wrapper.accounts.presale;
    let presale_endtime = wrapper
        .presale_params_wrapper
        .args
        .params
        .presale_params
        .presale_end_time;

    let q_price = calculate_q_price_from_ui_price(9.0f64, 9, 9);
    let fixed_price_args = &mut wrapper.fixed_point_params_wrapper.args.params;
    fixed_price_args.q_price = q_price;

    let presale_params = &mut wrapper.presale_params_wrapper.args.params.presale_params;
    presale_params.presale_minimum_cap = 92;
    presale_params.presale_maximum_cap = 100;

    let registry_0 = &mut wrapper
        .presale_params_wrapper
        .args
        .params
        .presale_registries
        .first_mut()
        .unwrap();

    registry_0.buyer_minimum_deposit_cap = 10;
    registry_0.buyer_maximum_deposit_cap = 100;

    let instructions = wrapper.to_instructions();
    process_transaction(&mut lite_svm, &instructions, Some(&user_pubkey), &[&user]).unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: 99,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    // This deposit will fail with zero token amount, making it impossible to reach the presale maximum cap if there's no gap between min and max cap
    let err = handle_escrow_deposit_err(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: 1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::ZeroTokenAmount;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));

    warp_time(&mut lite_svm, presale_endtime);
    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale_pubkey)
        .unwrap();

    let clock: Clock = lite_svm.get_sysvar();
    let presale_progress = presale_state.get_presale_progress(clock.unix_timestamp as u64);

    assert_eq!(presale_progress, PresaleProgress::Completed)
}

// https://www.notion.so/offsidelabs/Meteora-Presale-Audit-Draft-24dd5242e8af806f8703cdb86b093639#26bd5242e8af80d08ad3e750f85fa025
pub mod fixed_price_deposit_surplus_stuck_tests {
    use super::*;
    use presale::SCALE_MULTIPLIER;

    struct SetupResult {
        lite_svm: LiteSVM,
        user: Rc<Keypair>,
        buyer_minimum_deposit_cap: u64,
        presale_pubkey: Pubkey,
    }

    fn setup() -> SetupResult {
        let mut setup_context = SetupContext::initialize();
        let base_mint = setup_context.setup_mint(
            DEFAULT_BASE_TOKEN_DECIMALS,
            1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
        );

        let SetupContext { mut lite_svm, user } = setup_context;

        let user_pubkey = user.pubkey();
        let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
        let whitelist_mode = WhitelistMode::Permissionless;

        let mut wrapper = create_default_fixed_price_presale_args_wrapper(
            base_mint,
            quote_mint,
            &lite_svm,
            whitelist_mode,
            Rc::clone(&user),
            user_pubkey,
        );

        let presale_registries = &mut wrapper
            .presale_params_wrapper
            .args
            .params
            .presale_registries;

        let fixed_price_args = &wrapper.fixed_point_params_wrapper.args.params;

        let buyer_minimum_deposit_cap = {
            let presale_registry_args_0 = presale_registries.get_mut(0).unwrap();

            let token_lamport_price = fixed_price_args.q_price.div_ceil(SCALE_MULTIPLIER);
            let buyer_minimum_deposit_cap: u64 = token_lamport_price.try_into().unwrap();

            presale_registry_args_0.buyer_minimum_deposit_cap = buyer_minimum_deposit_cap;
            buyer_minimum_deposit_cap
        };

        let instructions = wrapper.to_instructions();

        assert!(
            process_transaction(&mut lite_svm, &instructions, Some(&user_pubkey), &[&user]).is_ok()
        );

        let presale_pubkey = derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID);

        SetupResult {
            lite_svm,
            user,
            buyer_minimum_deposit_cap,
            presale_pubkey,
        }
    }

    #[test]
    fn test_withdraw_fixed_price_with_suggested_amount() {
        let SetupResult {
            mut lite_svm,
            user,
            buyer_minimum_deposit_cap,
            presale_pubkey,
        } = setup();

        let user_pubkey = user.pubkey();
        let deposit_amount = buyer_minimum_deposit_cap * 2;

        handle_escrow_deposit(
            &mut lite_svm,
            HandleEscrowDepositArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                max_amount: deposit_amount,
                registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            },
        );

        handle_escrow_withdraw(
            &mut lite_svm,
            HandleEscrowWithdrawArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                amount: buyer_minimum_deposit_cap + buyer_minimum_deposit_cap / 2,
                registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            },
        );

        let escrow = derive_escrow(
            &presale_pubkey,
            &user_pubkey,
            DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            &presale::ID,
        );

        let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
        assert_eq!(escrow_state.total_deposit, buyer_minimum_deposit_cap);

        let presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();
        assert_eq!(presale_state.total_deposit, buyer_minimum_deposit_cap);
    }

    #[test]
    fn test_deposit_fixed_price_with_suggested_amount() {
        let SetupResult {
            mut lite_svm,
            user,
            buyer_minimum_deposit_cap,
            presale_pubkey,
        } = setup();

        let user_pubkey = user.pubkey();
        let deposit_amount = buyer_minimum_deposit_cap + buyer_minimum_deposit_cap / 2;

        handle_escrow_deposit(
            &mut lite_svm,
            HandleEscrowDepositArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                max_amount: deposit_amount,
                registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            },
        );

        let escrow = derive_escrow(
            &presale_pubkey,
            &user_pubkey,
            DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            &presale::ID,
        );

        let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
        assert_eq!(escrow_state.total_deposit, buyer_minimum_deposit_cap);

        let presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();
        assert_eq!(presale_state.total_deposit, buyer_minimum_deposit_cap);
    }

    #[test]
    fn test_deposit_fixed_price_with_stuck_surplus() {
        let SetupResult {
            mut lite_svm,
            user,
            buyer_minimum_deposit_cap,
            presale_pubkey,
        } = setup();

        let deposit_amount = buyer_minimum_deposit_cap - 1;

        let err = handle_escrow_deposit_err(
            &mut lite_svm,
            HandleEscrowDepositArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                max_amount: deposit_amount,
                registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            },
        );

        // Don't allow deposit because the surplus will stuck
        let expected_err = presale::errors::PresaleError::ZeroTokenAmount;
        let err_code = ERROR_CODE_OFFSET + expected_err as u32;
        let err_str = format!("Error Number: {}.", err_code);
        assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
    }

    #[test]
    fn test_withdraw_fixed_price_with_stuck_surplus() {
        let SetupResult {
            mut lite_svm,
            user,
            buyer_minimum_deposit_cap,
            presale_pubkey,
        } = setup();

        let user_pubkey = user.pubkey();

        handle_escrow_deposit(
            &mut lite_svm,
            HandleEscrowDepositArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                max_amount: buyer_minimum_deposit_cap,
                registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            },
        );

        let escrow = derive_escrow(
            &presale_pubkey,
            &user_pubkey,
            DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            &presale::ID,
        );

        let err = handle_escrow_withdraw_err(
            &mut lite_svm,
            HandleEscrowWithdrawArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                amount: 1,
                registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            },
        );

        // Don't allow withdraw because no tokens were bought
        let expected_err = presale::errors::PresaleError::ZeroTokenAmount;
        let err_code = ERROR_CODE_OFFSET + expected_err as u32;
        let err_str = format!("Error Number: {}.", err_code);
        assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));

        let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

        // But we can withdraw the full amount deposited
        handle_escrow_withdraw(
            &mut lite_svm,
            HandleEscrowWithdrawArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                amount: escrow_state.total_deposit,
                registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
            },
        );

        let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
        assert_eq!(escrow_state.total_deposit, 0);

        let presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();
        assert_eq!(presale_state.total_deposit, 0);
    }
}
