pub mod helpers;

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use anchor_lang::error::ERROR_CODE_OFFSET;
use anchor_lang::prelude::Clock;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::token_interface::TokenAccount;
use helpers::*;
use litesvm::LiteSVM;
use presale::{
    calculate_dripped_amount_for_user, Escrow, FixedPricePresaleHandler, Presale,
    DEFAULT_PERMISSIONLESS_REGISTRY_INDEX, SCALE_OFFSET,
};
use std::ops::Shl;
use std::rc::Rc;

enum Cmp {
    Equal,
    GreaterThan,
    #[allow(dead_code)]
    LessThan,
}

fn claim_and_assert(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
    presale_pubkey: Pubkey,
    registry_index: u8,
    cmp: Cmp,
    amount_delta: Option<u64>,
) {
    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let base_token_program = get_program_id_from_token_flag(presale_state.base_token_program_flag);

    let escrow = derive_escrow(
        &presale_pubkey,
        &user.pubkey(),
        registry_index,
        &presale::ID,
    );
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
            registry_index,
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

    if let Some(delta) = amount_delta {
        let amount_claimed = after_user_token_amount.saturating_sub(before_user_token_amount);
        assert_eq!(amount_claimed, delta);
    }
}

#[test]
fn test_claim_empty_prorata_presale_registries() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();

    let SetupContext { mut lite_svm, user } = setup_context;

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;
    let user_pubkey = user.pubkey();
    let user_1_pubkey = user_1.pubkey();

    let HandleCreatePredefinedPresaleResponse {  presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_merkle_proof_prorata_presale_with_multiple_presale_registries_refund_unsold(
            &mut lite_svm,
            mint,
            quote_mint,
            Rc::clone(&user));

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let whitelist_wallet = [
        WhitelistWallet {
            address: user_pubkey,
            registry_index: 0,
            max_deposit_cap: presale_state.presale_maximum_cap,
        },
        WhitelistWallet {
            address: user_1_pubkey,
            registry_index: 1,
            max_deposit_cap: presale_state.presale_maximum_cap,
        },
    ];

    let merkle_tree = build_merkle_tree(whitelist_wallet.to_vec(), 0);
    let merkle_root_config_address =
        merkle_tree.get_merkle_root_config_pubkey(presale_pubkey, &presale::ID);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    let tree_node_0 = merkle_tree.get_node(&user_pubkey);

    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            registry_index: tree_node_0.registry_index,
            merkle_root_config: merkle_root_config_address,
            max_deposit_cap: tree_node_0.deposit_cap,
            proof: tree_node_0.proof.unwrap(),
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: tree_node_0.deposit_cap,
            registry_index: tree_node_0.registry_index,
        },
    );

    let tree_node_1 = merkle_tree.get_node(&user_1_pubkey);

    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            registry_index: tree_node_1.registry_index,
            merkle_root_config: merkle_root_config_address,
            max_deposit_cap: tree_node_1.deposit_cap,
            proof: tree_node_1.proof.unwrap(),
        },
    );

    warp_time(&mut lite_svm, presale_state.vesting_end_time + 1);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        tree_node_0.registry_index,
        Cmp::GreaterThan,
        None,
    );

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        tree_node_1.registry_index,
        Cmp::Equal,
        None,
    );

    let escrow_1 = derive_escrow(
        &presale_pubkey,
        &user_1_pubkey,
        tree_node_1.registry_index,
        &presale::ID,
    );

    let escrow_state_1: Escrow = lite_svm.get_deserialized_zc_account(&escrow_1).unwrap();

    assert_eq!(escrow_state_1.total_deposit, 0);
}

#[test]
fn test_claim_with_immediate_release() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let SetupContext { mut lite_svm, user } = setup_context;

    let immediate_release_delta_from_presale_end = 60; // 1 minute after presale end

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale_with_immediate_release(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
            immediate_release_delta_from_presale_end,
        );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: LAMPORTS_PER_SOL,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_to_presale_end(&mut lite_svm, &presale_state);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::Equal,
        None,
    );

    warp_time(&mut lite_svm, presale_state.immediate_release_timestamp - 1);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::Equal,
        None,
    );

    warp_time(&mut lite_svm, presale_state.immediate_release_timestamp + 1);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );

    warp_time(&mut lite_svm, presale_state.immediate_release_timestamp + 2);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::Equal,
        None,
    );

    warp_time(&mut lite_svm, presale_state.vesting_end_time);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );
}

#[test]
fn test_claim_empty_escrow() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();

    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: LAMPORTS_PER_SOL,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.vesting_end_time + 1);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::Equal,
        None,
    );
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
            max_amount: LAMPORTS_PER_SOL / 2,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let err = handle_escrow_claim_err(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            refresh_escrow: true,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
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
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_to_presale_end(&mut lite_svm, &presale_state);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::Equal,
        None,
    );
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
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: amount_1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
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
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );
    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );

    warp_time(&mut lite_svm, presale_state.vesting_end_time);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::Equal,
        None,
    );
}

#[test]
fn test_claim_permissioned_fcfs_presale_with_multiple_presale_registries() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();
    let user_2 = setup_context.create_user();

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let SetupContext { mut lite_svm, user } = setup_context;

    let user_pubkey = user.pubkey();
    let user_1_pubkey = user_1.pubkey();
    let user_2_pubkey = user_2.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_merkle_proof_fcfs_presale_with_multiple_presale_registries(&mut lite_svm, mint, quote_mint, Rc::clone(&user));

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let whitelist_wallets = [
        WhitelistWallet {
            address: user_pubkey,
            registry_index: 0,
            max_deposit_cap: presale_state
                .presale_registries
                .get(0)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
        WhitelistWallet {
            address: user_1_pubkey,
            registry_index: 1,
            max_deposit_cap: presale_state
                .presale_registries
                .get(1)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
        WhitelistWallet {
            address: user_2_pubkey,
            registry_index: 1,
            max_deposit_cap: presale_state
                .presale_registries
                .get(1)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
    ];

    let merkle_tree = build_merkle_tree(whitelist_wallets.to_vec(), 0);
    let merkle_root_config =
        merkle_tree.get_merkle_root_config_pubkey(presale_pubkey, &presale::ID);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    let tree_node_0 = merkle_tree.get_node(&user_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_root_config,
            registry_index: tree_node_0.registry_index,
            proof: tree_node_0.proof.unwrap(),
            max_deposit_cap: tree_node_0.deposit_cap,
        },
    );

    let tree_node_1 = merkle_tree.get_node(&user_1_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            merkle_root_config,
            registry_index: tree_node_1.registry_index,
            proof: tree_node_1.proof.unwrap(),
            max_deposit_cap: tree_node_1.deposit_cap,
        },
    );

    let tree_node_2 = merkle_tree.get_node(&user_2_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_2),
            merkle_root_config,
            registry_index: tree_node_2.registry_index,
            proof: tree_node_2.proof.unwrap(),
            max_deposit_cap: tree_node_2.deposit_cap,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    // 3 users, weighted from light to heavy
    let total_weight = 1 + 2 + 3;
    let user_keypairs = [Rc::clone(&user), Rc::clone(&user_1), Rc::clone(&user_2)];

    for (idx, (wallet, keypair)) in whitelist_wallets
        .iter()
        .zip(user_keypairs.iter())
        .enumerate()
    {
        let weight = (idx + 1) as u64;
        let deposit_amount =
            (presale_state.presale_maximum_cap * weight + total_weight - 1) / total_weight;

        handle_escrow_deposit(
            &mut lite_svm,
            HandleEscrowDepositArgs {
                presale: presale_pubkey,
                owner: Rc::clone(keypair),
                max_amount: deposit_amount,
                registry_index: wallet.registry_index,
            },
        );
    }

    warp_time(
        &mut lite_svm,
        presale_state.vesting_end_time - presale_state.vest_duration / 2,
    );

    let clock: Clock = lite_svm.get_sysvar();

    for (wallet, keypair) in whitelist_wallets.iter().zip(user_keypairs.iter()) {
        let before_presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();

        let escrow = derive_escrow(
            &presale_pubkey,
            &wallet.address,
            wallet.registry_index,
            &presale::ID,
        );

        let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
        let presale_registry = before_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let amount_to_claim: u64 = calculate_dripped_amount_for_user(
            before_presale_state.vesting_start_time,
            before_presale_state.vest_duration,
            clock.unix_timestamp as u64,
            presale_registry.presale_supply,
            escrow_state.total_deposit,
            presale_registry.total_deposit,
        )
        .unwrap()
        .try_into()
        .unwrap();

        claim_and_assert(
            &mut lite_svm,
            Rc::clone(keypair),
            presale_pubkey,
            escrow_state.registry_index,
            Cmp::GreaterThan,
            Some(amount_to_claim),
        );

        let after_presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();

        let total_claim_delta = after_presale_state
            .total_claimed_token
            .saturating_sub(before_presale_state.total_claimed_token);

        assert_eq!(amount_to_claim, total_claim_delta);

        let before_presale_registry = before_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let after_presale_registry = after_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let total_claim_delta = after_presale_registry
            .total_claimed_token
            .saturating_sub(before_presale_registry.total_claimed_token);

        assert_eq!(total_claim_delta, amount_to_claim);
    }
}

#[test]
fn test_claim_permissioned_prorata_presale_with_multiple_presale_registries() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();
    let user_2 = setup_context.create_user();

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let SetupContext { mut lite_svm, user } = setup_context;

    let user_pubkey = user.pubkey();
    let user_1_pubkey = user_1.pubkey();
    let user_2_pubkey = user_2.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_merkle_proof_prorata_presale_with_multiple_presale_registries_refund_unsold(&mut lite_svm, mint, quote_mint, Rc::clone(&user));

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let whitelist_wallets = [
        WhitelistWallet {
            address: user_pubkey,
            registry_index: 0,
            max_deposit_cap: presale_state
                .presale_registries
                .get(0)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
        WhitelistWallet {
            address: user_1_pubkey,
            registry_index: 1,
            max_deposit_cap: presale_state
                .presale_registries
                .get(1)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
        WhitelistWallet {
            address: user_2_pubkey,
            registry_index: 1,
            max_deposit_cap: presale_state
                .presale_registries
                .get(1)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
    ];

    let merkle_tree = build_merkle_tree(whitelist_wallets.to_vec(), 0);
    let merkle_root_config =
        merkle_tree.get_merkle_root_config_pubkey(presale_pubkey, &presale::ID);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    let tree_node_0 = merkle_tree.get_node(&user_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_root_config,
            registry_index: tree_node_0.registry_index,
            proof: tree_node_0.proof.unwrap(),
            max_deposit_cap: tree_node_0.deposit_cap,
        },
    );

    let tree_node_1 = merkle_tree.get_node(&user_1_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            merkle_root_config,
            registry_index: tree_node_1.registry_index,
            proof: tree_node_1.proof.unwrap(),
            max_deposit_cap: tree_node_1.deposit_cap,
        },
    );

    let tree_node_2 = merkle_tree.get_node(&user_2_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_2),
            merkle_root_config,
            registry_index: tree_node_2.registry_index,
            proof: tree_node_2.proof.unwrap(),
            max_deposit_cap: tree_node_2.deposit_cap,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    // 3 users, weighted from light to heavy
    let total_weight = 1 + 2 + 3;
    let user_keypairs = [Rc::clone(&user), Rc::clone(&user_1), Rc::clone(&user_2)];

    for (idx, (wallet, keypair)) in whitelist_wallets
        .iter()
        .zip(user_keypairs.iter())
        .enumerate()
    {
        let weight = (idx + 1) as u64;
        let deposit_amount =
            (presale_state.presale_maximum_cap * weight + total_weight - 1) / total_weight;

        handle_escrow_deposit(
            &mut lite_svm,
            HandleEscrowDepositArgs {
                presale: presale_pubkey,
                owner: Rc::clone(keypair),
                max_amount: deposit_amount,
                registry_index: wallet.registry_index,
            },
        );
    }

    warp_time(
        &mut lite_svm,
        presale_state.vesting_end_time - presale_state.vest_duration / 2,
    );

    let clock: Clock = lite_svm.get_sysvar();

    for (wallet, keypair) in whitelist_wallets.iter().zip(user_keypairs.iter()) {
        let before_presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();

        let escrow = derive_escrow(
            &presale_pubkey,
            &wallet.address,
            wallet.registry_index,
            &presale::ID,
        );

        let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
        let presale_registry = before_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let amount_to_claim: u64 = calculate_dripped_amount_for_user(
            before_presale_state.vesting_start_time,
            before_presale_state.vest_duration,
            clock.unix_timestamp as u64,
            presale_registry.presale_supply,
            escrow_state.total_deposit,
            presale_registry.total_deposit,
        )
        .unwrap()
        .try_into()
        .unwrap();

        claim_and_assert(
            &mut lite_svm,
            Rc::clone(keypair),
            presale_pubkey,
            escrow_state.registry_index,
            Cmp::GreaterThan,
            Some(amount_to_claim),
        );

        let after_presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();

        let total_claim_delta = after_presale_state
            .total_claimed_token
            .saturating_sub(before_presale_state.total_claimed_token);

        assert_eq!(amount_to_claim, total_claim_delta);

        let before_presale_registry = before_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let after_presale_registry = after_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let total_claim_delta = after_presale_registry
            .total_claimed_token
            .saturating_sub(before_presale_registry.total_claimed_token);

        assert_eq!(total_claim_delta, amount_to_claim);
    }
}

#[test]
fn test_claim_permissioned_fixed_price_presale_with_multiple_presale_registries() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );

    let user_1 = setup_context.create_user();
    let user_2 = setup_context.create_user();

    let quote_mint = anchor_spl::token::spl_token::native_mint::ID;

    let SetupContext { mut lite_svm, user } = setup_context;

    let user_pubkey = user.pubkey();
    let user_1_pubkey = user_1.pubkey();
    let user_2_pubkey = user_2.pubkey();

    let HandleCreatePredefinedPresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale_with_multiple_presale_registries(&mut lite_svm, mint, quote_mint, Rc::clone(&user));

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let whitelist_wallets = [
        WhitelistWallet {
            address: user_pubkey,
            registry_index: 0,
            max_deposit_cap: presale_state
                .presale_registries
                .get(0)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
        WhitelistWallet {
            address: user_1_pubkey,
            registry_index: 1,
            max_deposit_cap: presale_state
                .presale_registries
                .get(1)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
        WhitelistWallet {
            address: user_2_pubkey,
            registry_index: 1,
            max_deposit_cap: presale_state
                .presale_registries
                .get(1)
                .unwrap()
                .buyer_maximum_deposit_cap,
        },
    ];

    let merkle_tree = build_merkle_tree(whitelist_wallets.to_vec(), 0);
    let merkle_root_config =
        merkle_tree.get_merkle_root_config_pubkey(presale_pubkey, &presale::ID);

    handle_create_merkle_root_config(
        &mut lite_svm,
        HandleCreateMerkleRootConfigArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_tree: &merkle_tree,
        },
    );

    let tree_node_0 = merkle_tree.get_node(&user_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            merkle_root_config,
            registry_index: tree_node_0.registry_index,
            proof: tree_node_0.proof.unwrap(),
            max_deposit_cap: tree_node_0.deposit_cap,
        },
    );

    let tree_node_1 = merkle_tree.get_node(&user_1_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            merkle_root_config,
            registry_index: tree_node_1.registry_index,
            proof: tree_node_1.proof.unwrap(),
            max_deposit_cap: tree_node_1.deposit_cap,
        },
    );

    let tree_node_2 = merkle_tree.get_node(&user_2_pubkey);
    handle_create_permissioned_escrow_with_merkle_proof(
        &mut lite_svm,
        HandleCreatePermissionedEscrowWithMerkleProofArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_2),
            merkle_root_config,
            registry_index: tree_node_2.registry_index,
            proof: tree_node_2.proof.unwrap(),
            max_deposit_cap: tree_node_2.deposit_cap,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    // 3 users, weighted from light to heavy
    let total_weight = 1 + 2 + 3;
    let user_keypairs = [Rc::clone(&user), Rc::clone(&user_1), Rc::clone(&user_2)];

    for (idx, (wallet, keypair)) in whitelist_wallets
        .iter()
        .zip(user_keypairs.iter())
        .enumerate()
    {
        let weight = (idx + 1) as u64;
        let deposit_amount =
            (presale_state.presale_maximum_cap * weight + total_weight - 1) / total_weight;

        handle_escrow_deposit(
            &mut lite_svm,
            HandleEscrowDepositArgs {
                presale: presale_pubkey,
                owner: Rc::clone(keypair),
                max_amount: deposit_amount,
                registry_index: wallet.registry_index,
            },
        );
    }

    warp_time(
        &mut lite_svm,
        presale_state.vesting_end_time - presale_state.vest_duration / 2,
    );

    let clock: Clock = lite_svm.get_sysvar();

    for (wallet, keypair) in whitelist_wallets.iter().zip(user_keypairs.iter()) {
        let before_presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();

        let escrow = derive_escrow(
            &presale_pubkey,
            &wallet.address,
            wallet.registry_index,
            &presale::ID,
        );

        let escrow_state: Escrow = lite_svm.get_deserialized_zc_account(&escrow).unwrap();
        let presale_registry = before_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let q_amount = u128::from(presale_registry.total_deposit).shl(SCALE_OFFSET);
        let fp_handler = decode_presale_mode_raw_data::<FixedPricePresaleHandler>(
            &presale_state.presale_mode_raw_data,
        );
        let registry_sold_token = q_amount / fp_handler.q_price;

        let amount_to_claim: u64 = calculate_dripped_amount_for_user(
            before_presale_state.vesting_start_time,
            before_presale_state.vest_duration,
            clock.unix_timestamp as u64,
            registry_sold_token.try_into().unwrap(),
            escrow_state.total_deposit,
            presale_registry.total_deposit,
        )
        .unwrap()
        .try_into()
        .unwrap();

        claim_and_assert(
            &mut lite_svm,
            Rc::clone(keypair),
            presale_pubkey,
            escrow_state.registry_index,
            Cmp::GreaterThan,
            Some(amount_to_claim),
        );

        let after_presale_state: Presale = lite_svm
            .get_deserialized_zc_account(&presale_pubkey)
            .unwrap();

        let total_claim_delta = after_presale_state
            .total_claimed_token
            .saturating_sub(before_presale_state.total_claimed_token);

        assert_eq!(amount_to_claim, total_claim_delta);

        let before_presale_registry = before_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let after_presale_registry = after_presale_state
            .presale_registries
            .get(escrow_state.registry_index as usize)
            .unwrap();

        let total_claim_delta = after_presale_registry
            .total_claimed_token
            .saturating_sub(before_presale_registry.total_claimed_token);

        assert_eq!(total_claim_delta, amount_to_claim);
    }
}

#[test]
fn test_claim_permissionless_presale() {
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
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: amount_1,
            registry_index: DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
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
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );
    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );

    warp_time(&mut lite_svm, presale_state.vesting_end_time);

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );
    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user_1),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::GreaterThan,
        None,
    );

    claim_and_assert(
        &mut lite_svm,
        Rc::clone(&user),
        presale_pubkey,
        DEFAULT_PERMISSIONLESS_REGISTRY_INDEX,
        Cmp::Equal,
        None,
    );
}
