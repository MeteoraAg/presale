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
    associated_token::get_associated_token_address_with_program_id, token_interface::Mint,
};
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use mpl_token_metadata::accounts::Metadata;
use presale::{
    AccountsType, LockedVestingArgs, PresaleArgs, PresaleMode, RemainingAccountsInfo,
    RemainingAccountsSlice, TokenomicArgs, UnsoldTokenAction, WhitelistMode,
};

pub fn create_tokenomic_args(decimals: u8) -> TokenomicArgs {
    TokenomicArgs {
        presale_pool_supply: 1_000_000_000 * 10u64.pow(decimals as u32), // 100 million with specified decimals
        ..Default::default()
    }
}

pub fn create_presale_args(lite_svm: &LiteSVM) -> PresaleArgs {
    let clock: Clock = lite_svm.get_sysvar();
    let presale_start_time = clock.unix_timestamp as u64;
    let presale_end_time = presale_start_time + 120; // 2 minutes later

    PresaleArgs {
        presale_start_time,
        presale_end_time,
        presale_maximum_cap: LAMPORTS_PER_SOL,
        presale_minimum_cap: 1_000_000, // 0.0001 SOL
        presale_mode: PresaleMode::FixedPrice.into(),
        buyer_maximum_deposit_cap: LAMPORTS_PER_SOL,
        buyer_minimum_deposit_cap: 1000,
        whitelist_mode: WhitelistMode::Permissionless.into(),
        ..Default::default()
    }
}

fn create_locked_vesting_args() -> LockedVestingArgs {
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
    pub tokenomic: TokenomicArgs,
    pub presale_params: PresaleArgs,
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
        tokenomic,
        presale_params,
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

    if base_mint_owner == anchor_spl::token::ID {
        let mint_metadata = Metadata::find_pda(&base_mint).0;
        accounts.push(AccountMeta {
            pubkey: mint_metadata,
            is_signer: false,
            is_writable: false,
        });
    }

    accounts.extend_from_slice(&remaining_accounts);

    let base_token_transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
        &base_mint_owner,
        &payer_presale_token,
        &base_mint,
        &presale_vault,
        &payer_pubkey,
        lite_svm,
    );

    let ix_data = presale::instruction::InitializePresale {
        params: presale::InitializePresaleArgs {
            tokenomic,
            presale_params,
            locked_vesting_params,
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

pub struct HandleCreatePredefinedPermissionlessFixedPricePresaleResponse {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub presale_pubkey: Pubkey,
}

pub fn handle_create_predefined_permissionless_fixed_price_presale(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPermissionlessFixedPricePresaleResponse {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();
    let quote_mint_account = lite_svm.get_account(&quote_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let quote_mint_state = Mint::try_deserialize(&mut quote_mint_account.data.as_ref())
        .expect("Failed to deserialize quote mint state");

    let user_pubkey = user.pubkey();

    let unsold_token_action = UnsoldTokenAction::Burn;
    let args = HandleInitializeFixedTokenPricePresaleParamsArgs {
        base_mint,
        quote_mint,
        q_price: calculate_q_price_from_ui_price(
            0.01,
            base_mint_state.decimals,
            quote_mint_state.decimals,
        ),
        unsold_token_action,
        owner: user_pubkey,
        payer: Rc::clone(&user),
        base: user_pubkey,
    };
    handle_initialize_fixed_token_price_presale_params(lite_svm, args.clone());

    let tokenomic = create_tokenomic_args(base_mint_state.decimals);

    let mut presale_params = create_presale_args(&lite_svm);
    presale_params.presale_mode = PresaleMode::FixedPrice.into();

    let locked_vesting_params = create_locked_vesting_args();

    handle_initialize_presale(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint,
            quote_mint,
            tokenomic,
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

    HandleCreatePredefinedPermissionlessFixedPricePresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}

pub struct HandleCreatePredefinedPermissionlessFcfsPresaleResponse {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub presale_pubkey: Pubkey,
}

pub fn handle_create_predefined_permissionless_fcfs_presale(
    lite_svm: &mut LiteSVM,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    user: Rc<Keypair>,
) -> HandleCreatePredefinedPermissionlessFcfsPresaleResponse {
    let base_mint_account = lite_svm.get_account(&base_mint).unwrap();

    let base_mint_state = Mint::try_deserialize(&mut base_mint_account.data.as_ref())
        .expect("Failed to deserialize base mint state");

    let user_pubkey = user.pubkey();

    let tokenomic = create_tokenomic_args(base_mint_state.decimals);

    let mut presale_params = create_presale_args(&lite_svm);
    presale_params.presale_mode = PresaleMode::Fcfs.into();

    let locked_vesting_params = create_locked_vesting_args();

    handle_initialize_presale(
        lite_svm,
        HandleInitializePresaleArgs {
            base_mint,
            quote_mint,
            tokenomic,
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

    HandleCreatePredefinedPermissionlessFcfsPresaleResponse {
        base_mint,
        quote_mint,
        presale_pubkey: derive_presale(&base_mint, &quote_mint, &user_pubkey, &presale::ID),
    }
}
