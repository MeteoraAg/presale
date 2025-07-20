use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use anchor_spl::associated_token::{
    get_associated_token_address_with_program_id,
    spl_associated_token_account::instruction::create_associated_token_account_idempotent,
};
use litesvm::LiteSVM;
use presale::{AccountsType, Presale, RemainingAccountsInfo, RemainingAccountsSlice};
use std::rc::Rc;

use crate::helpers::{
    derive_event_authority, get_extra_account_metas_for_transfer_hook,
    get_program_id_from_token_flag, process_transaction, LiteSVMExt,
};

pub struct HandlePerformUnsoldTokenActionArgs {
    pub presale: Pubkey,
    pub creator: Rc<Keypair>,
}

pub fn handle_perform_unsold_token_action(
    lite_svm: &mut LiteSVM,
    args: HandlePerformUnsoldTokenActionArgs,
) {
    let HandlePerformUnsoldTokenActionArgs { creator, presale } = args;

    let creator_pubkey = creator.pubkey();

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale)
        .unwrap();

    let token_program = get_program_id_from_token_flag(presale_state.base_token_program_flag);

    let creator_base_token = get_associated_token_address_with_program_id(
        &creator_pubkey,
        &presale_state.base_mint,
        &token_program,
    );

    let create_creator_base_token_ix = create_associated_token_account_idempotent(
        &creator_pubkey,
        &creator_pubkey,
        &presale_state.base_mint,
        &token_program,
    );

    let transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
        &token_program,
        &presale_state.base_token_vault,
        &presale_state.base_mint,
        &creator_base_token,
        &creator_pubkey,
        lite_svm,
    );

    let ix_data = presale::instruction::PerformUnsoldBaseTokenAction {
        remaining_accounts_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: transfer_hook_accounts.len() as u8,
            }],
        },
    }
    .data();

    let mut accounts = presale::accounts::PerformUnsoldBaseTokenActionCtx {
        presale,
        creator_base_token,
        event_authority: derive_event_authority(&presale::ID),
        token_program,
        program: presale::ID,
        base_mint: presale_state.base_mint,
        base_token_vault: presale_state.base_token_vault,
        presale_authority: presale::presale_authority::ID,
        memo_program: anchor_spl::memo::ID,
    }
    .to_account_metas(None);

    accounts.extend(transfer_hook_accounts);

    let ix = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    process_transaction(
        lite_svm,
        &[create_creator_base_token_ix, ix],
        Some(&creator_pubkey),
        &[&creator],
    )
    .unwrap();
}
