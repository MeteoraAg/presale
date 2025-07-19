use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::*;
use anchor_spl::associated_token::get_associated_token_address_with_program_id;
use litesvm::LiteSVM;
use presale::{AccountsType, Presale, RemainingAccountsInfo, RemainingAccountsSlice};
use std::rc::Rc;

use crate::helpers::{
    create_permissionless_escrow_ix, derive_escrow, derive_event_authority,
    get_extra_account_metas_for_transfer_hook, process_transaction,
    token::get_program_id_from_token_flag, LiteSVMExt,
};

pub struct HandleEscrowDepositArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
    pub max_amount: u64,
}

pub fn handle_escrow_deposit(lite_svm: &mut LiteSVM, args: HandleEscrowDepositArgs) {
    let HandleEscrowDepositArgs {
        owner,
        presale,
        max_amount,
    } = args;
    let owner_pubkey = owner.pubkey();

    let mut instructions = vec![];

    let create_permissionless_escrow_ix = create_permissionless_escrow_ix(
        lite_svm,
        super::HandleCreatePermissionlessEscrowArgs {
            presale,
            owner: Rc::clone(&owner),
        },
    );

    if let Some(ix) = create_permissionless_escrow_ix {
        instructions.push(ix);
    }

    let presale_state: Presale = lite_svm.get_deserialized_zc_account(&presale).unwrap();

    let quote_token_program = lite_svm
        .get_account(&presale_state.quote_mint)
        .unwrap()
        .owner;

    let escrow = derive_escrow(presale, owner_pubkey, &presale::ID);
    let payer_quote_token = get_associated_token_address_with_program_id(
        &owner_pubkey,
        &presale_state.quote_mint,
        &quote_token_program,
    );

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

    process_transaction(lite_svm, &instructions, Some(&owner_pubkey), &[&owner]).unwrap();
}
