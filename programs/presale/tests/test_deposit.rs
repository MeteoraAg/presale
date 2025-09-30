pub mod helpers;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use anchor_lang::{
    error::ERROR_CODE_OFFSET,
    prelude::{AccountMeta, Clock},
    AccountDeserialize,
};
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id,
    token_2022,
    token_interface::{Mint, TokenAccount},
};
use helpers::*;
use presale::{
    calculate_deposit_fee_included_amount, DepositFeeIncludedCalculation, Escrow, Presale,
    PresaleMode, PresaleProgress, PresaleRegistryArgs, Rounding, UnsoldTokenAction, WhitelistMode,
    DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
};
use std::rc::Rc;

#[test]
fn test_deposit_fixed_price_presale_progress_update() {
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

    let presale_state_0: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let clock: Clock = lite_svm.get_sysvar();
    let presale_progress_0 = presale_state_0.get_presale_progress(clock.unix_timestamp as u64);

    assert!(presale_progress_0 == PresaleProgress::Ongoing);

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            max_amount: presale_state_0.presale_maximum_cap,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state_1: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let clock: Clock = lite_svm.get_sysvar();
    let presale_progress_1 = presale_state_1.get_presale_progress(clock.unix_timestamp as u64);

    assert!(presale_progress_1 == PresaleProgress::Completed);
}

#[test]
fn test_deposit_fcfs_presale_progress_update() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fcfs_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let presale_state_0: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let clock: Clock = lite_svm.get_sysvar();
    let presale_progress_0 = presale_state_0.get_presale_progress(clock.unix_timestamp as u64);

    assert!(presale_progress_0 == PresaleProgress::Ongoing);

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            max_amount: presale_state_0.presale_maximum_cap,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state_1: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let clock: Clock = lite_svm.get_sysvar();
    let presale_progress_1 = presale_state_1.get_presale_progress(clock.unix_timestamp as u64);

    assert!(presale_progress_1 == PresaleProgress::Completed);
}

#[test]
fn test_deposit_below_buyer_minimum_cap() {
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

    let presale_registry = presale_state.get_presale_registry(0).unwrap();

    let deposit_amount = presale_registry.buyer_minimum_deposit_cap - 1;

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let err = handle_escrow_deposit_err(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::DepositAmountOutOfCap;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_deposit_before_presale_start() {
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

    let deposit_amount = LAMPORTS_PER_SOL / 2;

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_start_time - 1);

    let err = handle_escrow_deposit_err(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForDeposit;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_deposit_when_presale_ended() {
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

    let deposit_amount = LAMPORTS_PER_SOL / 2;

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_to_presale_end(&mut lite_svm, &presale_state);

    let err = handle_escrow_deposit_err(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let expected_err = presale::errors::PresaleError::PresaleNotOpenForDeposit;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err.meta.logs.iter().any(|log| log.contains(&err_str)));
}

#[test]
fn test_deposit_with_multiple_presale_registries_with_different_max_cap() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

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

    let operator = Rc::new(Keypair::new());
    let operator_pubkey = operator.pubkey();

    handle_create_operator(
        &mut lite_svm,
        HandleCreateOperatorArgs {
            owner: Rc::clone(&user),
            operator: operator_pubkey,
        },
    );

    let mut presale_registries = vec![];

    let registry_args_0 = PresaleRegistryArgs {
        buyer_minimum_deposit_cap: 100,
        buyer_maximum_deposit_cap: 600_000_000,
        presale_supply: 1_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
        ..PresaleRegistryArgs::default()
    };
    presale_registries.push(registry_args_0);

    let registry_args_1 = PresaleRegistryArgs {
        buyer_minimum_deposit_cap: 200,
        buyer_maximum_deposit_cap: 400_000_000,
        presale_supply: 2_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
        ..PresaleRegistryArgs::default()
    };
    presale_registries.push(registry_args_1);

    let instructions = custom_create_predefined_fixed_price_presale_ix(
        &mut lite_svm,
        mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithAuthority,
        UnsoldTokenAction::Refund,
        presale_registries.clone(),
        create_locked_vesting_args(),
    );

    process_transaction(&mut lite_svm, &instructions, Some(&user_pubkey), &[&user]).unwrap();

    let presale_pubkey = derive_presale(&mint, &quote_mint, &user_pubkey, &presale::ID);

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_create_permissioned_escrow_with_operator(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithOperatorArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            vault_owner: user_pubkey,
            operator: Rc::clone(&operator),
            registry_index: 0,
            max_deposit_cap: presale_state
                .presale_registries
                .get(0)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
    );

    handle_create_permissioned_escrow_with_operator(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithOperatorArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            vault_owner: user_pubkey,
            operator: Rc::clone(&operator),
            registry_index: 1,
            max_deposit_cap: presale_state
                .presale_registries
                .get(1)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
    );

    let registry_args_0 = presale_registries.get(0).unwrap();
    let registry_args_1 = presale_registries.get(1).unwrap();

    let before_presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: registry_args_0.buyer_maximum_deposit_cap * 2,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: registry_args_1.buyer_maximum_deposit_cap * 2,
            registry_index: 1,
        },
    );

    let after_presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let register_0 = after_presale_state.get_presale_registry(0).unwrap();
    assert_eq!(
        register_0.total_deposit,
        registry_args_0.buyer_maximum_deposit_cap
    );

    let register_1 = after_presale_state.get_presale_registry(1).unwrap();
    assert_eq!(
        register_1.total_deposit,
        registry_args_1.buyer_maximum_deposit_cap
    );

    // End presale earlier
    assert!(after_presale_state.presale_end_time < before_presale_state.presale_end_time);
    assert!(after_presale_state.lock_start_time < before_presale_state.lock_start_time);
    assert!(after_presale_state.lock_end_time < before_presale_state.lock_end_time);
    assert!(after_presale_state.vesting_start_time < before_presale_state.vesting_start_time);
    assert!(after_presale_state.vesting_end_time < before_presale_state.vesting_end_time);
}

#[test]
fn test_deposit() {
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

    let deposit_amount = LAMPORTS_PER_SOL / 2;

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

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(presale_state.total_deposit, deposit_amount);
    assert_eq!(escrow_state.total_deposit, deposit_amount);
}

#[test]
fn test_deposit_with_max_buyer_cap() {
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

    let deposit_amount = 10 * LAMPORTS_PER_SOL;

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

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let presale_registry = presale_state.get_presale_registry(0).unwrap();

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(
        presale_state.total_deposit,
        presale_registry.buyer_maximum_deposit_cap
    );
    assert_eq!(
        escrow_state.total_deposit,
        presale_registry.buyer_maximum_deposit_cap
    );
}

#[test]
fn test_deposit_with_max_presale_cap() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let user_1 = Rc::new(Keypair::new());
    let user_1_pubkey = user_1.pubkey();
    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        2 * LAMPORTS_PER_SOL,
    );

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    let before_presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let deposit_amount = LAMPORTS_PER_SOL / 2 + 100;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: deposit_amount,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let after_presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    assert_eq!(
        after_presale_state.total_deposit,
        after_presale_state.presale_maximum_cap
    );

    // End presale earlier
    assert!(after_presale_state.presale_end_time < before_presale_state.presale_end_time);
    assert!(after_presale_state.lock_start_time < before_presale_state.lock_start_time);
    assert!(after_presale_state.lock_end_time < before_presale_state.lock_end_time);
    assert!(after_presale_state.vesting_start_time < before_presale_state.vesting_start_time);
    assert!(after_presale_state.vesting_end_time < before_presale_state.vesting_end_time);
}

#[test]
fn test_deposit_over_escrow_max_deposit_cap() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;
    let user_pubkey = user.pubkey();

    let quote = anchor_spl::token::spl_token::native_mint::ID;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale_with_multiple_presale_registries(
            &mut lite_svm,
            mint,
            quote,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let presale_registry = presale_state.get_presale_registry(0).unwrap();
    let max_deposit_cap = (presale_registry.buyer_maximum_deposit_cap
        - presale_registry.buyer_minimum_deposit_cap)
        / 2;

    let whitelist_wallet = [WhitelistWallet {
        address: user_pubkey,
        registry_index: 0,
        max_deposit_cap,
    }];

    let merkle_tree = build_merkle_tree(whitelist_wallet.to_vec(), 0);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            merkle_tree: &merkle_tree,
            owner: Rc::clone(&user),
        },
    );

    let tree_node_0 = merkle_tree.get_node(&user_pubkey);

    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_root_config: merkle_tree
                .get_merkle_root_config_pubkey(presale_pubkey, &presale::ID),
            registry_index: tree_node_0.registry_index,
            proof: tree_node_0.proof.unwrap(),
            max_deposit_cap: tree_node_0.deposit_cap,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: tree_node_0.deposit_cap + 100,
            registry_index: tree_node_0.registry_index,
        },
    );

    let escrow_address = derive_escrow(
        &presale_pubkey,
        &user_pubkey,
        tree_node_0.registry_index,
        &presale::ID,
    );

    let escrow_state: Escrow = lite_svm
        .get_deserialized_zc_account(&escrow_address)
        .unwrap();

    assert_eq!(escrow_state.total_deposit, escrow_state.deposit_max_cap);
}

// Case: Deposit must within buyer min and max cap
// Someone might able to deposit, some might not
// presale_min_cap = 10, presale_max_cap = 100
// buyer_min_cap = 20, buyer_max_cap = 100
// user_0, max_deposit_cap = 70
// user_1, max_deposit_cap = 40
// user_2, max_deposit_cap = 30
// 1. user_0 deposit 70, presale.total_deposit = 70
// 2. user_1 deposit 20, presale.total_deposit = 90
// user_2 cannot deposit
// 3. user_1 deposit 10, presale.total_deposit = 100
#[test]
fn test_deposit_edge_case_0() {
    let mut setup_context = SetupContext::initialize();

    let user_1 = setup_context.create_user();
    let user_2 = setup_context.create_user();

    let base_mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let user_pubkey = user.pubkey();
    let user_1_pubkey = user_1.pubkey();
    let user_2_pubkey = user_2.pubkey();

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let quote_mint_state = Mint::try_deserialize(&mut quote_mint_account.data.as_ref())
        .expect("Failed to deserialize quote mint state");

    let args = HandleInitializeFixedTokenPricePresaleParamsArgs {
        base_mint,
        quote_mint,
        q_price: calculate_q_price_from_ui_price(
            DEFAULT_PRICE,
            base_mint_state.decimals,
            quote_mint_state.decimals,
        ),
        owner: user_pubkey,
        payer: Rc::clone(&user),
        base: user_pubkey,
    };
    let init_fixed_token_price_presale_args_ix =
        create_initialize_fixed_token_price_presale_params_args_ix(args.clone());

    let mut presale_params = create_presale_args(&lite_svm);
    presale_params.presale_minimum_cap = 10 * LAMPORTS_PER_SOL;
    presale_params.presale_maximum_cap = 100 * LAMPORTS_PER_SOL;
    presale_params.presale_mode = PresaleMode::FixedPrice.into();
    presale_params.whitelist_mode = WhitelistMode::PermissionWithMerkleProof.into();

    let locked_vesting_params = create_locked_vesting_args();

    let presale_registries = vec![PresaleRegistryArgs {
        buyer_minimum_deposit_cap: 20 * LAMPORTS_PER_SOL,
        buyer_maximum_deposit_cap: 100 * LAMPORTS_PER_SOL,
        presale_supply: 100_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
        ..PresaleRegistryArgs::default()
    }];

    let init_presale_ix = create_initialize_presale_ix(
        &lite_svm,
        HandleInitializePresaleArgs {
            base_mint,
            quote_mint,
            presale_registries,
            presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![AccountMeta {
                pubkey: derive_fixed_price_presale_args(
                    &base_mint,
                    &quote_mint,
                    &user_pubkey,
                    &presale::ID,
                ),
                is_signer: false,
                is_writable: false,
            }],
        },
    );

    let mut instructions = vec![];
    instructions.push(init_fixed_token_price_presale_args_ix);
    instructions.extend_from_slice(&init_presale_ix);

    process_transaction(&mut lite_svm, &instructions, Some(&user_pubkey), &[&user]).unwrap();

    let presale_pubkey = derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID);

    let whitelist_wallet = [
        WhitelistWallet {
            address: user_pubkey,
            registry_index: 0,
            max_deposit_cap: 70 * LAMPORTS_PER_SOL,
        },
        WhitelistWallet {
            address: user_1_pubkey,
            registry_index: 0,
            max_deposit_cap: 40 * LAMPORTS_PER_SOL,
        },
        WhitelistWallet {
            address: user_2_pubkey,
            registry_index: 0,
            max_deposit_cap: 30 * LAMPORTS_PER_SOL,
        },
    ];

    let merkle_tree = build_merkle_tree(whitelist_wallet.to_vec(), 0);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            merkle_tree: &merkle_tree,
            owner: Rc::clone(&user),
        },
    );

    let merkle_root_config =
        merkle_tree.get_merkle_root_config_pubkey(presale_pubkey, &presale::ID);

    for user in [&user, &user_1, &user_2] {
        let tree_node = merkle_tree.get_node(&user.pubkey());
        handle_create_permissioned_escrow_with_merkle_proof(
            &mut lite_svm,
            HandleCreatePermissionedEscrowWithMerkleProofArgs {
                presale: presale_pubkey,
                owner: Rc::clone(user),
                merkle_root_config,
                registry_index: tree_node.registry_index,
                proof: tree_node.proof.unwrap(),
                max_deposit_cap: tree_node.deposit_cap,
            },
        );
    }

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: 70 * LAMPORTS_PER_SOL,
            registry_index: 0,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: 20 * LAMPORTS_PER_SOL,
            registry_index: 0,
        },
    );

    let err_0 = handle_escrow_deposit_err(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_2),
            max_amount: 30 * LAMPORTS_PER_SOL,
            registry_index: 0,
        },
    );

    let expected_err = presale::errors::PresaleError::DepositAmountOutOfCap;
    let err_code = ERROR_CODE_OFFSET + expected_err as u32;
    let err_str = format!("Error Number: {}.", err_code);
    assert!(err_0.meta.logs.iter().any(|log| log.contains(&err_str)));

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: 30 * LAMPORTS_PER_SOL,
            registry_index: 0,
        },
    );

    let escrow_0 = derive_escrow(&presale_pubkey, &user_pubkey, 0, &presale::ID);
    let escrow_1 = derive_escrow(&presale_pubkey, &user_1_pubkey, 0, &presale::ID);
    let escrow_2 = derive_escrow(&presale_pubkey, &user_2_pubkey, 0, &presale::ID);

    let escrow_state_0: Escrow = lite_svm.get_deserialized_zc_account(&escrow_0).unwrap();
    assert_eq!(escrow_state_0.total_deposit, 70 * LAMPORTS_PER_SOL);

    let escrow_state_1: Escrow = lite_svm.get_deserialized_zc_account(&escrow_1).unwrap();
    assert_eq!(escrow_state_1.total_deposit, 30 * LAMPORTS_PER_SOL);

    let escrow_state_2: Escrow = lite_svm.get_deserialized_zc_account(&escrow_2).unwrap();
    assert_eq!(escrow_state_2.total_deposit, 0);
}

#[test]
fn test_deposit_2022_with_fee() {
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
    let user_pubkey = user.pubkey();
    let user_1_pubkey = user_1.pubkey();

    transfer_token(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        base_mint,
        100_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    transfer_token(
        &mut lite_svm,
        Rc::clone(&user),
        user_1_pubkey,
        quote_mint,
        100_000_000 * 10u64.pow(DEFAULT_QUOTE_TOKEN_DECIMALS.into()),
    );

    let presale_registries = vec![
        PresaleRegistryArgs {
            buyer_minimum_deposit_cap: 100,
            buyer_maximum_deposit_cap: 200_000_000,
            presale_supply: 1_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
            deposit_fee_bps: 100, // 1%
            ..PresaleRegistryArgs::default()
        },
        PresaleRegistryArgs {
            buyer_minimum_deposit_cap: 200,
            buyer_maximum_deposit_cap: 400_000_000,
            presale_supply: 2_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
            deposit_fee_bps: 200, // 2%
            ..PresaleRegistryArgs::default()
        },
    ];

    let create_ixs = custom_create_predefined_prorata_presale_ix(
        &mut lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
        presale_registries,
        create_locked_vesting_args(),
        UnsoldTokenAction::Refund,
    );

    process_transaction(&mut lite_svm, &create_ixs, Some(&user_pubkey), &[&user]).unwrap();

    let presale_pubkey = derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID);

    let whitelist_wallets = [
        WhitelistWallet {
            address: user_pubkey,
            registry_index: 0,
            max_deposit_cap: 200_000_000,
        },
        WhitelistWallet {
            address: user_1_pubkey,
            registry_index: 1,
            max_deposit_cap: 400_000_000,
        },
    ];

    let merkle_tree = build_merkle_tree(whitelist_wallets.to_vec(), 0);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    let deposit_amount_0 = 100_000_000;
    let deposit_amount_1 = 200_000_000;

    for (deposit_amount, user) in [
        (deposit_amount_0, Rc::clone(&user)),
        (deposit_amount_1, Rc::clone(&user_1)),
    ] {
        let tree_node = merkle_tree.get_node(&user.pubkey());
        handle_create_permissioned_escrow_with_merkle_proof(
            &mut lite_svm,
            HandleCreatePermissionedEscrowWithMerkleProofArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                merkle_root_config: merkle_tree
                    .get_merkle_root_config_pubkey(presale_pubkey, &presale::ID),
                registry_index: tree_node.registry_index,
                proof: tree_node.proof.unwrap(),
                max_deposit_cap: tree_node.deposit_cap,
            },
        );

        let user_quote_token_address = get_associated_token_address_with_program_id(
            &user.pubkey(),
            &quote_mint,
            &token_2022::ID,
        );

        let before_user_quote_token_account = lite_svm.get_account(&user_quote_token_address);

        handle_escrow_deposit(
            &mut lite_svm,
            HandleEscrowDepositArgs {
                presale: presale_pubkey,
                owner: Rc::clone(&user),
                max_amount: deposit_amount,
                registry_index: tree_node.registry_index,
            },
        );

        let after_user_quote_token_account =
            lite_svm.get_account(&user_quote_token_address).unwrap();

        let before_amount = if let Some(account) = before_user_quote_token_account {
            let token_account = TokenAccount::try_deserialize(&mut account.data.as_ref()).unwrap();
            token_account.amount
        } else {
            0
        };

        let after_amount =
            TokenAccount::try_deserialize(&mut after_user_quote_token_account.data.as_ref())
                .unwrap()
                .amount;

        let transfer_amount = before_amount - after_amount;

        let escrow = derive_escrow(
            &presale_pubkey,
            &user.pubkey(),
            tree_node.registry_index,
            &presale::ID,
        );

        let presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();

        let presale_registry = presale_state
            .get_presale_registry(tree_node.registry_index.into())
            .unwrap();

        let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

        let DepositFeeIncludedCalculation {
            fee,
            amount_included_fee,
        } = calculate_deposit_fee_included_amount(
            deposit_amount,
            presale_registry.deposit_fee_bps,
            Rounding::Up,
        )
        .unwrap();

        assert_eq!(escrow_state.total_deposit, deposit_amount);
        assert_eq!(escrow_state.total_deposit_fee, fee);
        assert_eq!(presale_registry.total_deposit, deposit_amount);
        assert_eq!(presale_registry.total_deposit_fee, fee);

        // Due to transfer fee
        assert!(transfer_amount > amount_included_fee);

        let total_deposit_in_registries = presale_state
            .presale_registries
            .iter()
            .fold(0u64, |acc, x| acc.checked_add(x.total_deposit).unwrap());

        let total_fee_in_registries = presale_state
            .presale_registries
            .iter()
            .fold(0u64, |acc, x| acc.checked_add(x.total_deposit_fee).unwrap());

        assert_eq!(total_deposit_in_registries, presale_state.total_deposit);
        assert_eq!(total_fee_in_registries, presale_state.total_deposit_fee);
    }
}

#[test]
fn test_deposit_token2022() {
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
    let user_pubkey = user.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            base_mint,
            quote_mint,
            Rc::clone(&user),
        );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let presale_registry = presale_state.get_presale_registry(0).unwrap();

    let deposit_amount = presale_registry.buyer_minimum_deposit_cap;

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

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let quote_token_vault = lite_svm
        .get_account(&presale_state.quote_token_vault)
        .unwrap();

    let quote_token = TokenAccount::try_deserialize(&mut quote_token_vault.data.as_ref()).unwrap();
    assert_eq!(quote_token.amount, deposit_amount);

    let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();

    assert_eq!(presale_state.total_deposit, deposit_amount);
    assert_eq!(escrow_state.total_deposit, deposit_amount);
}
