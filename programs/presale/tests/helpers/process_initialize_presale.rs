use std::rc::Rc;

use crate::helpers::*;
use anchor_client::solana_sdk::{
    instruction::Instruction, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair,
    signer::Signer,
};
use anchor_lang::{
    prelude::{AccountMeta, Clock},
    *,
};
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id,
    token_2022::spl_token_2022::extension::transfer_fee::MAX_FEE_BASIS_POINTS,
    token_interface::Mint,
};
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use presale::{
    AccountsType, LockedVestingArgs, PresaleArgs, PresaleMode, PresaleRegistryArgs,
    RemainingAccountsInfo, RemainingAccountsSlice, UnsoldTokenAction, WhitelistMode,
    MAX_PRESALE_REGISTRY_COUNT, SCALE_MULTIPLIER,
};

pub const PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS: [u16; 1] = [10_000];

pub const PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS: [u16; MAX_PRESALE_REGISTRY_COUNT] =
    [2_000, 2_000, 2_000, 2_000, 2_000];

pub const DEFAULT_DEPOSIT_BPS: u16 = 500;

pub const DEFAULT_PRICE: f64 = 0.01;

fn calculate_amount_by_bps(total_amount: u128, bps: u16) -> u128 {
    total_amount
        .checked_mul(bps.into())
        .unwrap()
        .checked_div(10_000)
        .unwrap()
}

pub fn create_default_presale_registries(
    base_decimals: u8,
    basis_points: &[u16],
    fixed_point_q_price: u128,
    whitelist_mode: WhitelistMode,
    presale_mode: PresaleMode,
    presale_max_cap: u64,
) -> Vec<PresaleRegistryArgs> {
    let mut presale_registries = vec![];
    for bps in basis_points.iter() {
        if *bps > 0 {
            let mut presale_registry = PresaleRegistryArgs::default();
            let presale_supply_lamport = 1_000_000_000_u128
                .checked_mul(10u128.pow(base_decimals.into()))
                .unwrap();
            let presale_supply = calculate_amount_by_bps(presale_supply_lamport, *bps);
            presale_registry.presale_supply = presale_supply.try_into().unwrap();

            if whitelist_mode.is_permissioned() {
                match presale_mode {
                    PresaleMode::FixedPrice => {
                        presale_registry.buyer_minimum_deposit_cap = fixed_point_q_price
                            .div_ceil(SCALE_MULTIPLIER)
                            .try_into()
                            .unwrap();
                    }
                    PresaleMode::Prorata | PresaleMode::Fcfs => {
                        presale_registry.buyer_minimum_deposit_cap = 1;
                    }
                }
                presale_registry.buyer_maximum_deposit_cap = presale_max_cap;
            } else {
                presale_registry.buyer_minimum_deposit_cap = 1_000;
                presale_registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
            }

            presale_registries.push(presale_registry);
        }
    }
    presale_registries
}

pub fn create_default_presale_args(lite_svm: &LiteSVM) -> PresaleArgs {
    let clock: Clock = lite_svm.get_sysvar();
    let presale_start_time = clock.unix_timestamp as u64;
    let presale_end_time = presale_start_time + 120; // 2 minutes later

    PresaleArgs {
        presale_start_time,
        presale_end_time,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        whitelist_mode: WhitelistMode::Permissionless.into(),
        unsold_token_action: UnsoldTokenAction::Refund.into(),
        ..Default::default()
    }
}

pub fn create_default_locked_vesting_args() -> LockedVestingArgs {
    LockedVestingArgs {
        lock_duration: 3600,  // 1 hour
        vest_duration: 86400, // 1 day
        ..Default::default()
    }
}

pub fn update_presale_registries_with_default_deposit_fee(
    presale_registries: &mut [PresaleRegistryArgs],
) {
    for presale_registry in presale_registries.iter_mut() {
        if presale_registry.is_uninitialized() {
            continue;
        }
        presale_registry.deposit_fee_bps = DEFAULT_DEPOSIT_BPS;
    }
}

pub fn build_initialize_presale_accounts(
    base_mint: Pubkey,
    quote_mint: Pubkey,
    base_mint_owner: Pubkey,
    quote_mint_owner: Pubkey,
    payer_pubkey: Pubkey,
    creator_pubkey: Pubkey,
) -> presale::accounts::InitializePresaleCtx {
    let presale = derive_presale(&base_mint, &quote_mint, &payer_pubkey, &presale::ID);
    let presale_vault = derive_presale_vault(&presale, &presale::ID);
    let quote_vault = derive_quote_vault(&presale, &presale::ID);
    let event_authority = derive_event_authority(&presale::ID);

    let payer_presale_token =
        get_associated_token_address_with_program_id(&payer_pubkey, &base_mint, &base_mint_owner);

    presale::accounts::InitializePresaleCtx {
        presale,
        presale_mint: base_mint,
        presale_authority: presale::presale_authority::ID,
        quote_token_mint: quote_mint,
        presale_vault,
        quote_token_vault: quote_vault,
        creator: creator_pubkey,
        payer: payer_pubkey,
        system_program: anchor_lang::solana_program::system_program::ID,
        event_authority,
        base_token_program: base_mint_owner,
        quote_token_program: quote_mint_owner,
        program: presale::ID,
        payer_presale_token,
        base: payer_pubkey,
    }
}

pub struct CreateDefaultPresaleArgsWrapper {
    pub args: presale::instruction::InitializePresale,
    pub accounts: presale::accounts::InitializePresaleCtx,
    pub remaining_accounts: Vec<AccountMeta>,
}

impl CreateDefaultPresaleArgsWrapper {
    pub fn to_instructions(self) -> Vec<Instruction> {
        let CreateDefaultPresaleArgsWrapper {
            args,
            accounts,
            remaining_accounts,
        } = self;

        let mut accounts = accounts.to_account_metas(None);
        accounts.extend_from_slice(&remaining_accounts);

        vec![Instruction {
            program_id: presale::ID,
            accounts,
            data: args.data(),
        }]
    }
}

pub struct CreateDefaultFixedPricePresaleArgsWrapper {
    pub presale_params_wrapper: CreateDefaultPresaleArgsWrapper,
    pub fixed_point_params_wrapper: CreateInitializeFixedTokenPricePresaleParamsArgsWrapper,
}

impl CreateDefaultFixedPricePresaleArgsWrapper {
    pub fn to_instructions(self) -> Vec<Instruction> {
        let CreateDefaultFixedPricePresaleArgsWrapper {
            presale_params_wrapper,
            fixed_point_params_wrapper,
        } = self;

        let CreateInitializeFixedTokenPricePresaleParamsArgsWrapper { accounts, args, .. } =
            fixed_point_params_wrapper;

        let init_fixed_price_params_ix = Instruction {
            program_id: presale::ID,
            accounts: accounts.to_account_metas(None),
            data: args.data(),
        };

        let init_presale_ix = presale_params_wrapper.to_instructions();
        let mut instructions = vec![init_fixed_price_params_ix];
        instructions.extend(init_presale_ix);

        instructions
    }
}

pub fn create_default_fcfs_presale_args_wrapper(
    base_mint: Pubkey,
    quote_mint: Pubkey,
    lite_svm: &LiteSVM,
    whitelist_mode: WhitelistMode,
    payer: Rc<Keypair>,
    creator_pubkey: Pubkey,
) -> CreateDefaultPresaleArgsWrapper {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let mut presale_args = create_default_presale_args(lite_svm);
    presale_args.presale_mode = PresaleMode::Fcfs.into();
    presale_args.whitelist_mode = whitelist_mode.into();

    let locked_vesting_args = create_default_locked_vesting_args();

    let payer_pubkey = payer.pubkey();

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
        0,
        whitelist_mode,
        presale_args.presale_mode.try_into().unwrap(),
        presale_args.presale_maximum_cap,
    );

    let accounts = build_initialize_presale_accounts(
        base_mint,
        quote_mint,
        base_mint_account.owner,
        quote_mint_account.owner,
        payer_pubkey,
        creator_pubkey,
    );

    let base_token_transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
        &base_mint_account.owner,
        &accounts.payer_presale_token,
        &base_mint,
        &accounts.presale_vault,
        &payer_pubkey,
        lite_svm,
    );

    let args = presale::instruction::InitializePresale {
        params: presale::InitializePresaleArgs {
            presale_registries,
            presale_params: presale_args,
            locked_vesting_params: locked_vesting_args,
            ..Default::default()
        },
        remaining_account_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: base_token_transfer_hook_accounts.len() as u8,
            }],
        },
    };

    CreateDefaultPresaleArgsWrapper {
        args,
        accounts,
        remaining_accounts: base_token_transfer_hook_accounts,
    }
}

pub fn create_default_prorata_presale_args_wrapper(
    base_mint: Pubkey,
    quote_mint: Pubkey,
    lite_svm: &LiteSVM,
    whitelist_mode: WhitelistMode,
    payer: Rc<Keypair>,
    creator_pubkey: Pubkey,
) -> CreateDefaultPresaleArgsWrapper {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let mut presale_args = create_default_presale_args(lite_svm);
    presale_args.presale_mode = PresaleMode::Prorata.into();
    presale_args.whitelist_mode = whitelist_mode.into();

    let locked_vesting_args = create_default_locked_vesting_args();

    let payer_pubkey = payer.pubkey();

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
        0,
        whitelist_mode,
        presale_args.presale_mode.try_into().unwrap(),
        presale_args.presale_maximum_cap,
    );

    let accounts = build_initialize_presale_accounts(
        base_mint,
        quote_mint,
        base_mint_account.owner,
        quote_mint_account.owner,
        payer_pubkey,
        creator_pubkey,
    );

    let base_token_transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
        &base_mint_account.owner,
        &accounts.payer_presale_token,
        &base_mint,
        &accounts.presale_vault,
        &payer_pubkey,
        lite_svm,
    );

    let args = presale::instruction::InitializePresale {
        params: presale::InitializePresaleArgs {
            presale_registries,
            presale_params: presale_args,
            locked_vesting_params: locked_vesting_args,
            ..Default::default()
        },
        remaining_account_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: base_token_transfer_hook_accounts.len() as u8,
            }],
        },
    };

    CreateDefaultPresaleArgsWrapper {
        args,
        accounts,
        remaining_accounts: base_token_transfer_hook_accounts,
    }
}

pub fn create_default_fixed_price_presale_args_wrapper(
    base_mint: Pubkey,
    quote_mint: Pubkey,
    lite_svm: &LiteSVM,
    whitelist_mode: WhitelistMode,
    payer: Rc<Keypair>,
    creator_pubkey: Pubkey,
) -> CreateDefaultFixedPricePresaleArgsWrapper {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let quote_mint_state = Mint::try_deserialize(&mut quote_mint_account.data.as_ref())
        .expect("Failed to deserialize quote mint state");

    let mut presale_args = create_default_presale_args(lite_svm);
    presale_args.presale_mode = PresaleMode::FixedPrice.into();
    presale_args.whitelist_mode = whitelist_mode.into();

    let locked_vesting_args = create_default_locked_vesting_args();

    let fixed_point_q_price = calculate_q_price_from_ui_price(
        DEFAULT_PRICE,
        base_mint_state.decimals,
        quote_mint_state.decimals,
    );

    let payer_pubkey = payer.pubkey();

    let fixed_point_params_wrapper =
        create_initialize_fixed_token_price_presale_params_args_wrapper(
            HandleInitializeFixedTokenPricePresaleParamsArgs {
                base_mint,
                quote_mint,
                q_price: fixed_point_q_price,
                owner: creator_pubkey,
                payer: Rc::clone(&payer),
                base: payer_pubkey,
                disable_withdraw: false,
            },
        );

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
        fixed_point_q_price,
        whitelist_mode,
        presale_args.presale_mode.try_into().unwrap(),
        presale_args.presale_maximum_cap,
    );

    let accounts = build_initialize_presale_accounts(
        base_mint,
        quote_mint,
        base_mint_account.owner,
        quote_mint_account.owner,
        payer_pubkey,
        creator_pubkey,
    );

    let fixed_price_args_pda =
        derive_fixed_price_presale_args(&base_mint, &quote_mint, &payer_pubkey, &presale::ID);

    let mut remaining_accounts = vec![AccountMeta::new_readonly(fixed_price_args_pda, false)];

    let base_token_transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
        &base_mint_account.owner,
        &accounts.payer_presale_token,
        &base_mint,
        &accounts.presale_vault,
        &payer_pubkey,
        lite_svm,
    );

    remaining_accounts.extend_from_slice(&base_token_transfer_hook_accounts);

    let args = presale::instruction::InitializePresale {
        params: presale::InitializePresaleArgs {
            presale_registries,
            presale_params: presale_args,
            locked_vesting_params: locked_vesting_args,
            ..Default::default()
        },
        remaining_account_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: base_token_transfer_hook_accounts.len() as u8,
            }],
        },
    };

    CreateDefaultFixedPricePresaleArgsWrapper {
        presale_params_wrapper: CreateDefaultPresaleArgsWrapper {
            args,
            accounts,
            remaining_accounts,
        },
        fixed_point_params_wrapper,
    }
}

pub struct HandleCreatePredefinedPresaleResponse {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub presale_pubkey: Pubkey,
}

pub fn create_predefined_fixed_price_presale_ix_with_immediate_release(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    immediate_release_delta_from_presale_end: i64,
) -> Vec<Instruction> {
    let mut wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );
    let CreateDefaultFixedPricePresaleArgsWrapper {
        presale_params_wrapper,
        ..
    } = &mut wrapper;

    let args = &mut presale_params_wrapper.args;
    let presale_args = &args.params.presale_params;
    let locked_vesting_args = &mut args.params.locked_vesting_params;

    locked_vesting_args.immediately_release_bps = 5000;
    locked_vesting_args.immediate_release_timestamp = i64::try_from(presale_args.presale_end_time)
        .unwrap()
        .checked_add(immediate_release_delta_from_presale_end)
        .unwrap()
        .try_into()
        .unwrap();

    wrapper.to_instructions()
}

pub fn create_predefined_fixed_price_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    wrapper.to_instructions()
}

pub fn create_predefined_fixed_price_presale_ix_with_deposit_fees(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let mut wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let CreateDefaultFixedPricePresaleArgsWrapper {
        presale_params_wrapper,
        ..
    } = &mut wrapper;

    let args = &mut presale_params_wrapper.args;
    let presale_registries = &mut args.params.presale_registries;
    update_presale_registries_with_default_deposit_fee(presale_registries);

    wrapper.to_instructions()
}

pub fn create_predefined_prorata_ix_with_no_vest_nor_lock(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let mut wrapper = create_default_prorata_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let locked_vesting_args = &mut wrapper.args.params.locked_vesting_params;
    locked_vesting_args.lock_duration = 0;
    locked_vesting_args.vest_duration = 0;
    locked_vesting_args.immediately_release_bps = MAX_FEE_BASIS_POINTS;

    wrapper.to_instructions()
}

pub fn create_predefined_fixed_price_presale_ix_with_multiple_registries(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let mut wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let CreateDefaultFixedPricePresaleArgsWrapper {
        presale_params_wrapper,
        fixed_point_params_wrapper,
    } = &mut wrapper;

    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_args = &presale_params_wrapper.args.params.presale_params;

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
        fixed_point_params_wrapper.args.params.q_price,
        presale_args.whitelist_mode.try_into().unwrap(),
        presale_args.presale_mode.try_into().unwrap(),
        presale_args.presale_maximum_cap,
    );

    presale_params_wrapper.args.params.presale_registries = presale_registries;

    wrapper.to_instructions()
}

fn create_predefined_prorata_presale_ix_with_deposit_fee(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let mut wrapper = create_default_prorata_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let presale_registries = &mut wrapper.args.params.presale_registries;
    update_presale_registries_with_default_deposit_fee(presale_registries);

    wrapper.to_instructions()
}

fn create_predefined_prorata_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let wrapper = create_default_prorata_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    wrapper.to_instructions()
}

fn create_predefined_prorata_presale_with_multiple_registries_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    unsold_token_action: UnsoldTokenAction,
) -> Vec<Instruction> {
    let mut wrapper = create_default_prorata_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_args = &wrapper.args.params.presale_params;

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
        0,
        presale_args.whitelist_mode.try_into().unwrap(),
        presale_args.presale_mode.try_into().unwrap(),
        presale_args.presale_maximum_cap,
    );

    wrapper.args.params.presale_registries = presale_registries;
    wrapper.args.params.presale_params.unsold_token_action = unsold_token_action.into();
    wrapper.to_instructions()
}

fn create_predefined_fcfs_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let wrapper = create_default_fcfs_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    wrapper.to_instructions()
}

fn create_predefined_fcfs_presale_ix_with_deposit_fee(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let mut wrapper = create_default_fcfs_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let presale_registries = &mut wrapper.args.params.presale_registries;
    update_presale_registries_with_default_deposit_fee(presale_registries);

    wrapper.to_instructions()
}

fn create_predefined_fcfs_presale_with_multiple_registries_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let mut wrapper = create_default_fcfs_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_args = &wrapper.args.params.presale_params;

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
        0,
        presale_args.whitelist_mode.try_into().unwrap(),
        presale_args.presale_mode.try_into().unwrap(),
        presale_args.presale_maximum_cap,
    );

    wrapper.args.params.presale_registries = presale_registries;
    wrapper.to_instructions()
}

pub fn handle_initialize_presale(lite_svm: &mut LiteSVM, args: HandleInitializePresaleArgs) {
    let HandleInitializePresaleArgs { payer, .. } = args.clone();
    let instructions = create_initialize_presale_ix(lite_svm, args);
    process_transaction(lite_svm, &instructions, Some(&payer.pubkey()), &[&payer]).unwrap();
}

pub fn handle_initialize_presale_err(
    lite_svm: &mut LiteSVM,
    args: HandleInitializePresaleArgs,
) -> FailedTransactionMetadata {
    let HandleInitializePresaleArgs { payer, .. } = args.clone();
    let instructions = create_initialize_presale_ix(lite_svm, args);
    process_transaction(lite_svm, &instructions, Some(&payer.pubkey()), &[&payer]).unwrap_err()
}

pub fn handle_create_predefined_permissionless_prorata_presale(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_prorata_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissionless_fcfs_presale(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fcfs_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissionless_fcfs_presale_with_deposit_fees(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fcfs_presale_ix_with_deposit_fee(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissionless_fixed_price_presale(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fixed_price_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissionless_fixed_price_presale_with_immediate_release(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    immediate_release_delta_from_presale_end: i64,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fixed_price_presale_ix_with_immediate_release(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
        immediate_release_delta_from_presale_end,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissionless_prorata_presale_with_deposit_fee(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_prorata_presale_ix_with_deposit_fee(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissionless_fixed_price_presale_with_deposit_fee(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fixed_price_presale_ix_with_deposit_fees(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissionless_prorata_presale_with_no_vest_nor_lock(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_prorata_ix_with_no_vest_nor_lock(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissioned_with_authority_fixed_price_presale(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fixed_price_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithAuthority,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fixed_price_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissioned_with_merkle_proof_fixed_price_presale_with_multiple_presale_registries(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fixed_price_presale_ix_with_multiple_registries(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissioned_with_merkle_proof_prorata_presale_with_multiple_presale_registries_refund_unsold(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_prorata_presale_with_multiple_registries_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
        UnsoldTokenAction::Refund,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissioned_with_merkle_proof_prorata_presale_with_multiple_presale_registries_burn_unsold(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_prorata_presale_with_multiple_registries_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
        UnsoldTokenAction::Burn,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub fn handle_create_predefined_permissioned_with_merkle_proof_fcfs_presale_with_multiple_presale_registries(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fcfs_presale_with_multiple_registries_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
    );

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

#[derive(Clone)]
pub struct HandleInitializePresaleArgs {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub presale_params: PresaleArgs,
    pub presale_registries: Vec<PresaleRegistryArgs>,
    pub locked_vesting_params: Option<LockedVestingArgs>,
    pub creator: Pubkey,
    pub payer: Rc<Keypair>,
    pub remaining_accounts: Vec<AccountMeta>,
}

pub fn create_initialize_presale_ix(
    lite_svm: &LiteSVM,
    args: HandleInitializePresaleArgs,
) -> Vec<Instruction> {
    let HandleInitializePresaleArgs {
        base_mint,
        quote_mint,
        presale_params,
        presale_registries,
        locked_vesting_params,
        creator,
        payer,
        remaining_accounts,
    } = args;

    let payer_pubkey = payer.pubkey();

    let presale = derive_presale(&base_mint, &quote_mint, &payer_pubkey, &presale::ID);
    let presale_vault = derive_presale_vault(&presale, &presale::ID);
    let quote_vault = derive_quote_vault(&presale, &presale::ID);
    let event_authority = derive_event_authority(&presale::ID);

    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_owner = base_mint_account.owner;
    let quote_mint_owner = quote_mint_account.owner;

    let payer_presale_token =
        get_associated_token_address_with_program_id(&payer_pubkey, &base_mint, &base_mint_owner);

    let mut accounts = presale::accounts::InitializePresaleCtx {
        presale,
        presale_mint: base_mint,
        presale_authority: presale::presale_authority::ID,
        quote_token_mint: quote_mint,
        presale_vault,
        quote_token_vault: quote_vault,
        creator,
        payer: payer_pubkey,
        system_program: anchor_lang::solana_program::system_program::ID,
        event_authority,
        base_token_program: base_mint_owner,
        quote_token_program: quote_mint_owner,
        program: presale::ID,
        payer_presale_token,
        base: payer_pubkey,
    }
    .to_account_metas(None);

    accounts.extend_from_slice(&remaining_accounts);

    let base_token_transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
        &base_mint_owner,
        &payer_presale_token,
        &base_mint,
        &presale_vault,
        &payer_pubkey,
        lite_svm,
    );

    accounts.extend_from_slice(&base_token_transfer_hook_accounts);

    let ix_data = presale::instruction::InitializePresale {
        params: presale::InitializePresaleArgs {
            presale_registries,
            presale_params,
            locked_vesting_params: locked_vesting_params.unwrap_or_default(),
            ..Default::default()
        },
        remaining_account_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: base_token_transfer_hook_accounts.len() as u8,
            }],
        },
    }
    .data();

    let init_presale_ix = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    vec![init_presale_ix]
}
