use std::rc::Rc;

use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::{instruction::Instruction, signer::Signer};
use anchor_lang::prelude::Pubkey;
use anchor_lang::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use litesvm::types::FailedTransactionMetadata;
use litesvm::LiteSVM;
use presale::{AccountsType, Presale, RemainingAccountsInfo, RemainingAccountsSlice};

use crate::helpers::{
    derive_event_authority, get_extra_account_metas_for_transfer_hook, process_transaction,
    LiteSVMExt,
};

#[derive(Clone)]
pub struct HandleCreatorCollectFeeArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

fn create_creator_collect_fee_ix(
    lite_svm: &mut LiteSVM,
    args: HandleCreatorCollectFeeArgs,
) -> Vec<Instruction> {
    let HandleCreatorCollectFeeArgs { presale, owner } = args;
    let owner_pubkey = owner.pubkey();

    let presale_state: Presale = lite_svm.get_deserialized_zc_account(&presale).unwrap();

    let quote_mint_account = lite_svm.get_account(&presale_state.quote_mint).unwrap();

    let owner_quote_token_address = get_associated_token_address_with_program_id(
        &owner_pubkey,
        &presale_state.quote_mint,
        &quote_mint_account.owner,
    );

    let create_owner_quote_token_ix = create_associated_token_account_idempotent(
        &owner_pubkey,
        &owner_pubkey,
        &presale_state.quote_mint,
        &quote_mint_account.owner,
    );

    let transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
        &quote_mint_account.owner,
        &presale_state.quote_token_vault,
        &presale_state.quote_mint,
        &owner_quote_token_address,
        &owner_pubkey,
        lite_svm,
    );

    let ix_data = presale::instruction::CreatorCollectFee {
        remaining_accounts_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookQuote,
                length: transfer_hook_accounts.len() as u8,
            }],
        },
    }
    .data();

    let mut accounts = presale::accounts::CreatorCollectFeeCtx {
        presale,
        owner: owner_pubkey,
        quote_mint: presale_state.quote_mint,
        quote_token_vault: presale_state.quote_token_vault,
        memo_program: anchor_spl::memo::ID,
        presale_authority: presale::presale_authority::ID,
        fee_receiving_account: owner_quote_token_address,
        token_program: quote_mint_account.owner,
        event_authority: derive_event_authority(&presale::ID),
        program: presale::ID,
    }
    .to_account_metas(None);

    accounts.extend_from_slice(&transfer_hook_accounts);

    let collect_fee_ix = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    vec![create_owner_quote_token_ix, collect_fee_ix]
}

pub fn handle_creator_collect_fee(lite_svm: &mut LiteSVM, args: HandleCreatorCollectFeeArgs) {
    let instructions = create_creator_collect_fee_ix(lite_svm, args.clone());
    let HandleCreatorCollectFeeArgs { owner, .. } = args;
    process_transaction(lite_svm, &instructions, Some(&owner.pubkey()), &[&owner]).unwrap();
}

pub fn handle_creator_collect_fee_err(
    lite_svm: &mut LiteSVM,
    args: HandleCreatorCollectFeeArgs,
) -> FailedTransactionMetadata {
    let instructions = create_creator_collect_fee_ix(lite_svm, args.clone());
    let HandleCreatorCollectFeeArgs { owner, .. } = args;
    process_transaction(lite_svm, &instructions, Some(&owner.pubkey()), &[&owner]).unwrap_err()
}
