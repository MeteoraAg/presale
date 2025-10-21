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
    MAX_PRESALE_REGISTRY_COUNT,
};

pub const PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS: [u16; 1] = [10_000];

pub const PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS: [u16; MAX_PRESALE_REGISTRY_COUNT] =
    [2_000, 2_000, 2_000, 2_000, 2_000];

pub const DEFAULT_DEPOSIT_BPS: u16 = 500;

pub const DEFAULT_PRICE: f64 = 0.01;

pub fn create_default_presale_registries(
    decimals: u8,
    basis_points: &[u16],
) -> Vec<PresaleRegistryArgs> {
    let mut presale_registries = vec![];
    for bps in basis_points.iter() {
        if *bps > 0 {
            let mut presale_registry = PresaleRegistryArgs::default();
            let presale_supply =
                1_000_000_000u128 * 10u128.pow(decimals.into()) * u128::from(*bps) / 10_000u128;
            presale_registry.presale_supply = presale_supply.try_into().unwrap();
            presale_registry.buyer_maximum_deposit_cap = LAMPORTS_PER_SOL;
            presale_registry.buyer_minimum_deposit_cap = 1_000;
            presale_registries.push(presale_registry);
        }
    }
    presale_registries
}

pub fn create_default_presale_registries_with_deposit_fee(
    decimals: u8,
    basis_points: &[u16],
) -> Vec<PresaleRegistryArgs> {
    let mut presale_registries = create_default_presale_registries(decimals, basis_points);

    for presale_registry in presale_registries.iter_mut() {
        presale_registry.deposit_fee_bps = DEFAULT_DEPOSIT_BPS;
    }

    presale_registries
}

pub fn get_default_presale_start_and_end_time(lite_svm: &LiteSVM) -> (u64, u64) {
    let clock: Clock = lite_svm.get_sysvar();
    let presale_start_time = clock.unix_timestamp as u64;
    let presale_end_time = presale_start_time + 120; // 2 minutes later

    (presale_start_time, presale_end_time)
}

pub fn create_presale_args(lite_svm: &LiteSVM) -> PresaleArgs {
    let (presale_start_time, presale_end_time) = get_default_presale_start_and_end_time(lite_svm);

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

pub fn create_locked_vesting_args() -> LockedVestingArgs {
    LockedVestingArgs {
        lock_duration: 3600,  // 1 hour
        vest_duration: 86400, // 1 day
        ..Default::default()
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

pub struct HandleCreatePredefinedPresaleResponse {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub presale_pubkey: Pubkey,
}

#[derive(Default)]
pub struct CustomCreatePredefinedFixedPricePresaleIxArgs {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub whitelist_mode: WhitelistMode,
    pub disable_withdraw: bool,
    pub disable_presale_end_earlier: bool,
    pub unsold_token_action: UnsoldTokenAction,
    pub presale_registries: Vec<PresaleRegistryArgs>,
    pub locked_vesting_args: LockedVestingArgs,
}

pub fn custom_create_predefined_fixed_price_presale_ix(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
    args: CustomCreatePredefinedFixedPricePresaleIxArgs,
) -> Vec<Instruction> {
    let CustomCreatePredefinedFixedPricePresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        disable_withdraw,
        disable_presale_end_earlier,
        unsold_token_action,
        presale_registries,
        locked_vesting_args,
    } = args;

    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let quote_mint_state = Mint::try_deserialize(&mut quote_mint_account.data.as_ref())
        .expect("Failed to deserialize quote mint state");

    let user_pubkey = user.pubkey();

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
        disable_withdraw,
    };
    let init_fixed_token_price_presale_args_ix =
        create_initialize_fixed_token_price_presale_params_args_ix(args.clone());

    let mut presale_params = create_presale_args(lite_svm);
    presale_params.unsold_token_action = unsold_token_action.into();
    presale_params.presale_mode = PresaleMode::FixedPrice.into();
    presale_params.whitelist_mode = whitelist_mode.into();
    presale_params.disable_earlier_presale_end_once_cap_reached =
        u8::from(disable_presale_end_earlier);

    let init_presale_ix = create_initialize_presale_ix(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint,
            quote_mint,
            presale_registries,
            presale_params,
            locked_vesting_params: Some(locked_vesting_args),
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

    [
        vec![init_fixed_token_price_presale_args_ix],
        init_presale_ix,
    ]
    .concat()
}

pub fn create_predefined_fixed_price_presale_ix_with_immediate_release(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    unsold_token_action: UnsoldTokenAction,
    immediate_release_delta_from_presale_end: i64,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let mut locked_vesting_args = create_locked_vesting_args();
    locked_vesting_args.immediately_release_bps = 5000;
    let (_presale_start_time, presale_end_time) = get_default_presale_start_and_end_time(lite_svm);
    locked_vesting_args.immediate_release_timestamp = (presale_end_time as i64
        + immediate_release_delta_from_presale_end)
        .try_into()
        .unwrap();

    let args = CustomCreatePredefinedFixedPricePresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        unsold_token_action,
        presale_registries,
        locked_vesting_args,
        ..Default::default()
    };

    custom_create_predefined_fixed_price_presale_ix(lite_svm, user, args)
}

pub fn create_predefined_fixed_price_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    unsold_token_action: UnsoldTokenAction,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let args = CustomCreatePredefinedFixedPricePresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        unsold_token_action,
        presale_registries,
        locked_vesting_args: create_locked_vesting_args(),
        ..Default::default()
    };

    custom_create_predefined_fixed_price_presale_ix(lite_svm, user, args)
}

pub fn create_predefined_fixed_price_presale_ix_with_deposit_fees(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    unsold_token_action: UnsoldTokenAction,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries_with_deposit_fee(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let args = CustomCreatePredefinedFixedPricePresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        unsold_token_action,
        presale_registries,
        locked_vesting_args: create_locked_vesting_args(),
        ..Default::default()
    };

    custom_create_predefined_fixed_price_presale_ix(lite_svm, user, args)
}

pub fn create_predefined_fixed_prorata_ix_with_no_vest_nor_lock(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries_with_deposit_fee(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    custom_create_predefined_prorata_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        user,
        whitelist_mode,
        presale_registries,
        LockedVestingArgs {
            immediately_release_bps: MAX_FEE_BASIS_POINTS,
            ..Default::default()
        },
        UnsoldTokenAction::Refund,
    )
}

pub fn create_predefined_fixed_price_presale_ix_with_multiple_registries(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    unsold_token_action: UnsoldTokenAction,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let args = CustomCreatePredefinedFixedPricePresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        unsold_token_action,
        presale_registries,
        locked_vesting_args: create_locked_vesting_args(),
        ..Default::default()
    };

    custom_create_predefined_fixed_price_presale_ix(lite_svm, user, args)
}

pub fn custom_create_predefined_prorata_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    presale_registries: Vec<PresaleRegistryArgs>,
    locked_vesting_params: LockedVestingArgs,
    unsold_token_action: UnsoldTokenAction,
) -> Vec<Instruction> {
    let user_pubkey = user.pubkey();

    let mut presale_params = create_presale_args(lite_svm);
    presale_params.presale_mode = PresaleMode::Prorata.into();
    presale_params.whitelist_mode = whitelist_mode.into();
    presale_params.unsold_token_action = unsold_token_action.into();

    create_initialize_presale_ix(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint,
            quote_mint,
            presale_registries,
            presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    )
}

fn create_predefined_prorata_presale_ix_with_deposit_fee(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries_with_deposit_fee(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    custom_create_predefined_prorata_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        user,
        whitelist_mode,
        presale_registries,
        create_locked_vesting_args(),
        UnsoldTokenAction::Refund,
    )
}

fn create_predefined_prorata_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    custom_create_predefined_prorata_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        user,
        whitelist_mode,
        presale_registries,
        create_locked_vesting_args(),
        UnsoldTokenAction::Refund,
    )
}

fn create_predefined_prorata_presale_with_multiple_registries_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
    unsold_token_action: UnsoldTokenAction,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    custom_create_predefined_prorata_presale_ix(
        lite_svm,
        base_mint,
        quote_mint,
        user,
        whitelist_mode,
        presale_registries,
        create_locked_vesting_args(),
        unsold_token_action,
    )
}

#[derive(Default)]
pub struct CustomCreatePredefinedFcfsPresaleIxArgs {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub whitelist_mode: WhitelistMode,
    pub disable_presale_end_earlier: bool,
    pub presale_registries: Vec<PresaleRegistryArgs>,
}

pub fn custom_create_predefined_fcfs_presale_ix(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
    args: CustomCreatePredefinedFcfsPresaleIxArgs,
) -> Vec<Instruction> {
    let user_pubkey = user.pubkey();

    let CustomCreatePredefinedFcfsPresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        presale_registries,
        disable_presale_end_earlier,
    } = args;

    let mut presale_params = create_presale_args(lite_svm);
    presale_params.presale_mode = PresaleMode::Fcfs.into();
    presale_params.whitelist_mode = whitelist_mode.into();
    presale_params.disable_earlier_presale_end_once_cap_reached =
        u8::from(disable_presale_end_earlier);

    let locked_vesting_params = create_locked_vesting_args();

    create_initialize_presale_ix(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint,
            quote_mint,
            presale_registries,
            presale_params,
            locked_vesting_params: Some(locked_vesting_params),
            creator: user_pubkey,
            payer: Rc::clone(&user),
            remaining_accounts: vec![],
        },
    )
}

fn create_predefined_fcfs_presale_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let args = CustomCreatePredefinedFcfsPresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        presale_registries,
        ..Default::default()
    };

    custom_create_predefined_fcfs_presale_ix(lite_svm, user, args)
}

fn create_predefined_fcfs_presale_ix_with_deposit_fee(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries_with_deposit_fee(
        base_mint_state.decimals,
        &PRESALE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let args = CustomCreatePredefinedFcfsPresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        presale_registries,
        ..Default::default()
    };

    custom_create_predefined_fcfs_presale_ix(lite_svm, user, args)
}

fn create_predefined_fcfs_presale_with_multiple_registries_ix(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
    whitelist_mode: WhitelistMode,
) -> Vec<Instruction> {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let presale_registries = create_default_presale_registries(
        base_mint_state.decimals,
        &PRESALE_MULTIPLE_REGISTRIES_DEFAULT_BASIS_POINTS,
    );

    let args = CustomCreatePredefinedFcfsPresaleIxArgs {
        base_mint,
        quote_mint,
        whitelist_mode,
        presale_registries,
        ..Default::default()
    };

    custom_create_predefined_fcfs_presale_ix(lite_svm, user, args)
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
        UnsoldTokenAction::Burn,
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

pub fn handle_create_predefined_permissionless_prorata_presale_with_no_vest_nor_lock(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPresaleResponse {
    let instructions = create_predefined_fixed_prorata_ix_with_no_vest_nor_lock(
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
