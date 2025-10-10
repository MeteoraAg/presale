use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    system_instruction,
};
use anchor_lang::*;
use anchor_spl::{
    associated_token::{
        get_associated_token_address_with_program_id,
        spl_associated_token_account::instruction::create_associated_token_account_idempotent,
    },
    token_2022::spl_token_2022::instruction::sync_native,
};
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use presale::{AccountsType, Presale, RemainingAccountsInfo, RemainingAccountsSlice};
use std::rc::Rc;

use crate::helpers::{
    create_permissionless_escrow_ix, derive_escrow, derive_event_authority,
    get_extra_account_metas_for_transfer_hook, process_transaction,
    token::get_program_id_from_token_flag, LiteSVMExt,
};

#[derive(Clone)]
pub struct HandleEscrowDepositArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
    pub max_amount: u64,
    pub registry_index: u8,
}

pub fn create_deposit_ix(
    lite_svm: &mut LiteSVM,
    args: HandleEscrowDepositArgs,
) -> Vec<Instruction> {
    let HandleEscrowDepositArgs {
        owner,
        presale,
        max_amount,
        registry_index,
    } = args;
    let owner_pubkey = owner.pubkey();

    let mut instructions = vec![];

    let create_permissionless_escrow_ix = create_permissionless_escrow_ix(
        lite_svm,
        super::HandleCreatePermissionlessEscrowArgs {
            presale,
            owner: Rc::clone(&owner),
            registry_index,
        },
    );

    if let Some(ix) = create_permissionless_escrow_ix {
        instructions.push(ix);
    }

    let presale_state: Presale = lite_svm.get_deserialized_zc_account(&presale).unwrap();
    let presale_registry = presale_state
        .get_presale_registry(registry_index.into())
        .unwrap();

    let quote_token_program = lite_svm
        .get_account(&presale_state.quote_mint)
        .unwrap()
        .owner;

    let escrow = derive_escrow(&presale, &owner_pubkey, registry_index, &presale::ID);

    let payer_quote_token = get_associated_token_address_with_program_id(
        &owner_pubkey,
        &presale_state.quote_mint,
        &quote_token_program,
    );

    let create_payer_quote_token_ix = create_associated_token_account_idempotent(
        &owner_pubkey,
        &owner_pubkey,
        &presale_state.quote_mint,
        &quote_token_program,
    );
    instructions.push(create_payer_quote_token_ix);

    if presale_state.quote_mint == anchor_spl::token::spl_token::native_mint::ID {
        let deposit_fee_included_max_amount = presale_registry
            .calculate_deposit_fee_included_amount(max_amount)
            .unwrap();

        instructions.push(system_instruction::transfer(
            &owner.pubkey(),
            &payer_quote_token,
            deposit_fee_included_max_amount.amount_included_fee,
        ));

        instructions.push(sync_native(&quote_token_program, &payer_quote_token).unwrap());
    }

    let transfer_hook_account = get_extra_account_metas_for_transfer_hook(
        &get_program_id_from_token_flag(presale_state.quote_token_program_flag),
        &payer_quote_token,
        &presale_state.quote_mint,
        &presale_state.quote_token_vault,
        &owner.pubkey(),
        lite_svm,
    );

    let ix_data = presale::instruction::Deposit {
        max_amount,
        remaining_account_info: RemainingAccountsInfo {
            slices: vec![RemainingAccountsSlice {
                accounts_type: AccountsType::TransferHookQuote,
                length: transfer_hook_account.len() as u8,
            }],
        },
    }
    .data();

    let mut accounts = presale::accounts::DepositCtx {
        quote_mint: presale_state.quote_mint,
        quote_token_vault: presale_state.quote_token_vault,
        payer_quote_token,
        escrow,
        token_program: quote_token_program,
        program: presale::ID,
        presale,
        payer: owner.pubkey(),
        event_authority: derive_event_authority(&presale::ID),
    }
    .to_account_metas(None);

    accounts.extend(transfer_hook_account);

    let instruction = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };
    instructions.push(instruction);

    instructions
}

pub fn handle_escrow_deposit(lite_svm: &mut LiteSVM, args: HandleEscrowDepositArgs) {
    let instructions = create_deposit_ix(lite_svm, args.clone());
    let HandleEscrowDepositArgs { owner, .. } = args;
    process_transaction(lite_svm, &instructions, Some(&owner.pubkey()), &[&owner]).unwrap();
}

pub fn handle_escrow_deposit_err(
    lite_svm: &mut LiteSVM,
    args: HandleEscrowDepositArgs,
) -> FailedTransactionMetadata {
    let instructions = create_deposit_ix(lite_svm, args.clone());
    let HandleEscrowDepositArgs { owner, .. } = args;
    process_transaction(lite_svm, &instructions, Some(&owner.pubkey()), &[&owner]).unwrap_err()
}
