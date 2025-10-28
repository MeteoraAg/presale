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
    AccountsType, CommonPresaleArgs, LockedVestingArgs, PresaleArgs, PresaleMode,
    PresaleRegistryArgs, RemainingAccountsInfo, RemainingAccountsSlice, UnsoldTokenAction,
    WhitelistMode, MAX_PRESALE_REGISTRY_COUNT, SCALE_MULTIPLIER,
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
        whitelist_mode: WhitelistMode::Permissionless.into(),
        unsold_token_action: UnsoldTokenAction::Refund.into(),
        ..Default::default()
    }
}

pub fn create_default_locked_vesting_args(presale_end_time: u64) -> LockedVestingArgs {
    LockedVestingArgs {
        lock_duration: 3600,  // 1 hour
        vest_duration: 86400, // 1 day
        immediate_release_timestamp: presale_end_time,
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

fn to_instruction(
    args: &impl InstructionData,
    accounts: &impl ToAccountMetas,
    remaining_account: Vec<AccountMeta>,
) -> Instruction {
    let mut accounts = accounts.to_account_metas(None);
    accounts.extend(remaining_account);

    Instruction {
        program_id: presale::ID,
        accounts,
        data: args.data(),
    }
}

pub struct CreateDefaultProrataPresaleArgsWrapper {
    pub args: presale::instruction::InitializeProrataPresale,
    pub accounts: presale::accounts::InitializePresaleCtx,
    pub remaining_accounts: Vec<AccountMeta>,
}

impl CreateDefaultProrataPresaleArgsWrapper {
    pub fn to_instruction(&self) -> Instruction {
        to_instruction(&self.args, &self.accounts, self.remaining_accounts.clone())
    }
}

pub struct CreateDefaultFixedPricePresaleArgsWrapper {
    pub args: presale::instruction::InitializeFixedPricePresale,
    pub accounts: presale::accounts::InitializePresaleCtx,
    pub remaining_accounts: Vec<AccountMeta>,
}

impl CreateDefaultFixedPricePresaleArgsWrapper {
    pub fn to_instruction(&self) -> Instruction {
        to_instruction(&self.args, &self.accounts, self.remaining_accounts.clone())
    }
}

pub struct CreateDefaultFcfsPresaleArgsWrapper {
    pub args: presale::instruction::InitializeFcfsPresale,
    pub accounts: presale::accounts::InitializePresaleCtx,
    pub remaining_accounts: Vec<AccountMeta>,
}

impl CreateDefaultFcfsPresaleArgsWrapper {
    pub fn to_instruction(&self) -> Instruction {
        to_instruction(&self.args, &self.accounts, self.remaining_accounts.clone())
    }
}

pub fn create_default_fcfs_presale_args_wrapper(
    base_mint: Pubkey,
    quote_mint: Pubkey,
    lite_svm: &LiteSVM,
    whitelist_mode: WhitelistMode,
    payer: Rc<Keypair>,
    creator_pubkey: Pubkey,
) -> CreateDefaultFcfsPresaleArgsWrapper {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let mut presale_args = create_default_presale_args(lite_svm);
    presale_args.whitelist_mode = whitelist_mode.into();

    let locked_vesting_args = create_default_locked_vesting_args(presale_args.presale_end_time);

    let payer_pubkey = payer.pubkey();

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
        0,
        whitelist_mode,
        PresaleMode::Fcfs,
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

    let args = presale::instruction::InitializeFcfsPresale {
        params: presale::InitializeFcfsPresaleArgs {
            common_args: CommonPresaleArgs {
                presale_registries,
                presale_params: presale_args,
                locked_vesting_params: locked_vesting_args,
                ..Default::default()
            },
            disable_earlier_presale_end_once_cap_reached: false.into(),
            ..Default::default()
        },
        remaining_account_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: base_token_transfer_hook_accounts.len() as u8,
            }],
        },
    };

    CreateDefaultFcfsPresaleArgsWrapper {
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
) -> CreateDefaultProrataPresaleArgsWrapper {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let mut presale_args = create_default_presale_args(lite_svm);
    presale_args.whitelist_mode = whitelist_mode.into();

    let locked_vesting_args = create_default_locked_vesting_args(presale_args.presale_end_time);

    let payer_pubkey = payer.pubkey();

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
        0,
        whitelist_mode,
        PresaleMode::Prorata,
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

    let args = presale::instruction::InitializeProrataPresale {
        params: presale::InitializeProrataPresaleArgs {
            common_args: CommonPresaleArgs {
                presale_registries,
                presale_params: presale_args,
                locked_vesting_params: locked_vesting_args,
                ..Default::default()
            },
            ..Default::default()
        },
        remaining_account_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: base_token_transfer_hook_accounts.len() as u8,
            }],
        },
    };

    CreateDefaultProrataPresaleArgsWrapper {
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
    presale_args.whitelist_mode = whitelist_mode.into();

    let locked_vesting_args = create_default_locked_vesting_args(presale_args.presale_end_time);

    let fixed_point_q_price = calculate_q_price_from_ui_price(
        DEFAULT_PRICE,
        base_mint_state.decimals,
        quote_mint_state.decimals,
    );

    let payer_pubkey = payer.pubkey();

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
        fixed_point_q_price,
        whitelist_mode,
        PresaleMode::FixedPrice,
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

    let remaining_accounts = get_extra_account_metas_for_transfer_hook(
        &base_mint_account.owner,
        &accounts.payer_presale_token,
        &base_mint,
        &accounts.presale_vault,
        &payer_pubkey,
        lite_svm,
    );

    let args = presale::instruction::InitializeFixedPricePresale {
        params: presale::InitializeFixedPricePresaleArgs {
            common_args: CommonPresaleArgs {
                presale_registries,
                presale_params: presale_args,
                locked_vesting_params: locked_vesting_args,
                ..Default::default()
            },
            q_price: fixed_point_q_price,
            disable_earlier_presale_end_once_cap_reached: false.into(),
            disable_withdraw: false.into(),
            ..Default::default()
        },
        remaining_account_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: remaining_accounts.len() as u8,
            }],
        },
    };

    CreateDefaultFixedPricePresaleArgsWrapper {
        args,
        accounts,
        remaining_accounts,
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
) -> Instruction {
    let mut wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );
    let CreateDefaultFixedPricePresaleArgsWrapper { args, .. } = &mut wrapper;

    let common_args = &mut args.params.common_args;
    let presale_args = &common_args.presale_params;
    let locked_vesting_args = &mut common_args.locked_vesting_params;

    locked_vesting_args.immediately_release_bps = 5000;
    locked_vesting_args.immediate_release_timestamp = i64::try_from(presale_args.presale_end_time)
        .unwrap()
        .checked_add(immediate_release_delta_from_presale_end)
        .unwrap()
        .try_into()
        .unwrap();

    wrapper.to_instruction()
}

pub fn create_predefined_fixed_price_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
    let wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    wrapper.to_instruction()
}

pub fn create_predefined_fixed_price_presale_ix_with_deposit_fees(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
    let mut wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let CreateDefaultFixedPricePresaleArgsWrapper { args, .. } = &mut wrapper;

    let presale_registries = &mut args.params.common_args.presale_registries;
    update_presale_registries_with_default_deposit_fee(presale_registries);

    wrapper.to_instruction()
}

pub fn create_predefined_prorata_ix_with_no_vest_nor_lock(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
    let mut wrapper = create_default_prorata_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let locked_vesting_args = &mut wrapper.args.params.common_args.locked_vesting_params;
    locked_vesting_args.lock_duration = 0;
    locked_vesting_args.vest_duration = 0;
    locked_vesting_args.immediately_release_bps = MAX_FEE_BASIS_POINTS;

    wrapper.to_instruction()
}

pub fn create_predefined_fixed_price_presale_ix_with_multiple_registries(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
    let mut wrapper = create_default_fixed_price_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let CreateDefaultFixedPricePresaleArgsWrapper { args, .. } = &mut wrapper;

    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_args = &args.params.common_args.presale_params;

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
        args.params.q_price,
        presale_args.whitelist_mode.into(),
        PresaleMode::FixedPrice,
        presale_args.presale_maximum_cap,
    );

    args.params.common_args.presale_registries = presale_registries;

    wrapper.to_instruction()
}

fn create_predefined_prorata_presale_ix_with_deposit_fee(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
    let mut wrapper = create_default_prorata_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let presale_registries = &mut wrapper.args.params.common_args.presale_registries;
    update_presale_registries_with_default_deposit_fee(presale_registries);

    wrapper.to_instruction()
}

fn create_predefined_prorata_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
    let wrapper = create_default_prorata_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    wrapper.to_instruction()
}

fn create_predefined_prorata_presale_with_multiple_registries_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    unsold_token_action: UnsoldTokenAction,
) -> Instruction {
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

    let presale_args = &wrapper.args.params.common_args.presale_params;

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
        0,
        presale_args.whitelist_mode.into(),
        PresaleMode::Prorata,
        presale_args.presale_maximum_cap,
    );

    let common_args = &mut wrapper.args.params.common_args;

    common_args.presale_registries = presale_registries;
    common_args.presale_params.unsold_token_action = unsold_token_action.into();

    wrapper.to_instruction()
}

fn create_predefined_fcfs_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
    let wrapper = create_default_fcfs_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    wrapper.to_instruction()
}

fn create_predefined_fcfs_presale_ix_with_deposit_fee(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
    let mut wrapper = create_default_fcfs_presale_args_wrapper(
        base_mint,
        quote_mint,
        lite_svm,
        whitelist_mode,
        Rc::clone(&user),
        user.pubkey(),
    );

    let presale_registries = &mut wrapper.args.params.common_args.presale_registries;
    update_presale_registries_with_default_deposit_fee(presale_registries);

    wrapper.to_instruction()
}

fn create_predefined_fcfs_presale_with_multiple_registries_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Instruction {
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

    let presale_args = &wrapper.args.params.common_args.presale_params;

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
        0,
        presale_args.whitelist_mode.into(),
        PresaleMode::Fcfs,
        presale_args.presale_maximum_cap,
    );

    wrapper.args.params.common_args.presale_registries = presale_registries;
    wrapper.to_instruction()
}

pub fn handle_create_predefined_permissionless_prorata_presale(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instruction = create_predefined_prorata_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fcfs_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fcfs_presale_ix_with_deposit_fee(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fixed_price_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fixed_price_presale_ix_with_immediate_release(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
        immediate_release_delta_from_presale_end,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_prorata_presale_ix_with_deposit_fee(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fixed_price_presale_ix_with_deposit_fees(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_prorata_ix_with_no_vest_nor_lock(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::Permissionless,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fixed_price_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithAuthority,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fixed_price_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fixed_price_presale_ix_with_multiple_registries(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_prorata_presale_with_multiple_registries_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
        UnsoldTokenAction::Refund,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_prorata_presale_with_multiple_registries_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
        UnsoldTokenAction::Burn,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

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
    let instruction = create_predefined_fcfs_presale_with_multiple_registries_ix(
        lite_svm,
        base_mint,
        quote_mint,
        Rc::clone(&user),
        WhitelistMode::PermissionWithMerkleProof,
    );

    process_transaction(lite_svm, &[instruction], Some(&user.pubkey()), &[&user]).unwrap();

    let user_pubkey = user.pubkey();

    HandleCreatePredefinedPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

// pub fn handle_initialize_presale(lite_svm: &mut LiteSVM, args: HandleInitializePresaleArgs) {
//     let HandleInitializePresaleArgs { payer, .. } = args.clone();
//     let instructions = create_initialize_presale_ix(lite_svm, args);
//     process_transaction(lite_svm, &instructions, Some(&payer.pubkey()), &[&payer]).unwrap();
// }

// pub fn handle_initialize_presale_err(
//     lite_svm: &mut LiteSVM,
//     args: HandleInitializePresaleArgs,
// ) -> FailedTransactionMetadata {
//     let HandleInitializePresaleArgs { payer, .. } = args.clone();
//     let instructions = create_initialize_presale_ix(lite_svm, args);
//     process_transaction(lite_svm, &instructions, Some(&payer.pubkey()), &[&payer]).unwrap_err()
// }
