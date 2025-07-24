use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use anchor_spl::associated_token::{
    get_associated_token_address_with_program_id,
    spl_associated_token_account::instruction::create_associated_token_account_idempotent,
};
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use presale::{AccountsType, Presale, RemainingAccountsInfo, RemainingAccountsSlice};
use std::rc::Rc;

use crate::helpers::{
    derive_escrow, derive_event_authority, get_extra_account_metas_for_transfer_hook,
    process_transaction, LiteSVMExt,
};

#[derive(Clone)]
pub struct HandleEscrowClaimArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
    pub refresh_escrow: bool,
}

pub fn create_escrow_claim_ix(
    lite_svm: &mut LiteSVM,
    args: HandleEscrowClaimArgs,
) -> Vec<Instruction> {
    let HandleEscrowClaimArgs {
        owner,
        presale,
        refresh_escrow,
    } = args;

    let owner_pubkey = owner.pubkey();
    let escrow = derive_escrow(&presale, &owner_pubkey, &presale::ID);

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale)
        .unwrap();

    let token_program = lite_svm
        .get_account(&presale_state.base_mint)
        .unwrap()
        .owner;

    let owner_base_token = get_associated_token_address_with_program_id(
        &owner.pubkey(),
        &presale_state.base_mint,
        &token_program,
    );

    let create_owner_base_token_ix = create_associated_token_account_idempotent(
        &owner.pubkey(),
        &owner.pubkey(),
        &presale_state.base_mint,
        &token_program,
    );

    let transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
        &token_program,
        &presale_state.base_token_vault,
        &presale_state.base_mint,
        &owner_base_token,
        &owner_pubkey,
        lite_svm,
    );

    let ix_data = presale::instruction::Claim {
        remaining_accounts_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookBase,
                length: transfer_hook_accounts.len() as u8,
            }],
        },
    }
    .data();

    let mut accounts = presale::accounts::ClaimCtx {
        presale,
        escrow,
        owner: owner_pubkey,
        event_authority: derive_event_authority(&presale::ID),
        token_program,
        program: presale::ID,
        base_mint: presale_state.base_mint,
        base_token_vault: presale_state.base_token_vault,
        presale_authority: presale::presale_authority::ID,
        owner_base_token,
        memo_program: anchor_spl::memo::ID,
    }
    .to_account_metas(None);

    accounts.extend(transfer_hook_accounts);

    let claim_ix = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    let ix_data = presale::instruction::RefreshEscrow {}.data();

    let accounts = presale::accounts::RefreshEscrowCtx {
        presale,
        escrow,
        program: presale::ID,
        event_authority: derive_event_authority(&presale::ID),
    }
    .to_account_metas(None);

    let refresh_ix = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    if refresh_escrow {
        vec![create_owner_base_token_ix, refresh_ix, claim_ix]
    } else {
        vec![create_owner_base_token_ix, claim_ix]
    }
}

pub fn handle_escrow_claim(lite_svm: &mut LiteSVM, args: HandleEscrowClaimArgs) {
    let instructions = create_escrow_claim_ix(lite_svm, args.clone());
    let owner = Rc::clone(&args.owner);
    let owner_pubkey = owner.pubkey();
    process_transaction(lite_svm, &instructions, Some(&owner_pubkey), &[&owner]).unwrap();
}

pub fn handle_escrow_claim_err(
    lite_svm: &mut LiteSVM,
    args: HandleEscrowClaimArgs,
) -> FailedTransactionMetadata {
    let instructions = create_escrow_claim_ix(lite_svm, args.clone());
    let owner = Rc::clone(&args.owner);
    let owner_pubkey = owner.pubkey();
    process_transaction(lite_svm, &instructions, Some(&owner_pubkey), &[&owner]).unwrap_err()
}
