use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::{
    prelude::{AccountMeta, Clock},
    *,
};
use anchor_spl::associated_token::{
    get_associated_token_address_with_program_id,
    spl_associated_token_account::instruction::create_associated_token_account_idempotent,
};
use litesvm::LiteSVM;
use presale::{
    AccountsType, Presale, PresaleProgress, RemainingAccountsInfo, RemainingAccountsSlice,
};
use std::rc::Rc;

use crate::helpers::{
    derive_event_authority, get_extra_account_metas_for_transfer_hook,
    get_program_id_from_token_flag, process_transaction, LiteSVMExt,
};

// TODO: Handle presale progress
pub struct HandleCreatorWithdrawTokenArgs {
    pub presale: Pubkey,
    pub owner: Rc<Keypair>,
}

pub fn handle_creator_withdraw_token(lite_svm: &mut LiteSVM, args: HandleCreatorWithdrawTokenArgs) {
    let HandleCreatorWithdrawTokenArgs { owner, presale } = args;

    let owner_pubkey = owner.pubkey();

    let presale_state = lite_svm
        .get_deserialized_zc_account::<Presale>(&presale)
        .unwrap();

    let clock: Clock = lite_svm.get_sysvar();
    let presale_progress = presale_state.get_presale_progress(clock.unix_timestamp as u64);

    let (
        owner_token,
        create_owner_token_ix,
        token_vault,
        mint,
        remaining_accounts_info,
        token_program,
        transfer_hook_accounts,
    ) = if presale_progress == PresaleProgress::Completed {
        let token_program = get_program_id_from_token_flag(presale_state.quote_token_program_flag);
        let creator_quote_token = get_associated_token_address_with_program_id(
            &owner.pubkey(),
            &presale_state.quote_mint,
            &token_program,
        );

        let create_owner_quote_token_ix = create_associated_token_account_idempotent(
            &owner.pubkey(),
            &owner.pubkey(),
            &presale_state.quote_mint,
            &token_program,
        );

        let transfer_hook_accounts = get_extra_account_metas_for_transfer_hook(
            &token_program,
            &presale_state.quote_token_vault,
            &presale_state.quote_mint,
            &creator_quote_token,
            &owner_pubkey,
            lite_svm,
        );

        (
            creator_quote_token,
            create_owner_quote_token_ix,
            presale_state.quote_token_vault,
            presale_state.quote_mint,
            RemainingAccountsInfo {
                slices: vec![RemainingAccountsSlice {
                    accounts_type: AccountsType::TransferHookQuote,
                    length: transfer_hook_accounts.len() as u8,
                }],
            },
            token_program,
            transfer_hook_accounts,
        )
    } else {
        let token_program = get_program_id_from_token_flag(presale_state.base_token_program_flag);
        let creator_base_token = get_associated_token_address_with_program_id(
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
            &creator_base_token,
            &owner_pubkey,
            lite_svm,
        );

        (
            creator_base_token,
            create_owner_base_token_ix,
            presale_state.base_token_vault,
            presale_state.base_mint,
            RemainingAccountsInfo {
                slices: vec![RemainingAccountsSlice {
                    accounts_type: AccountsType::TransferHookBase,
                    length: transfer_hook_accounts.len() as u8,
                }],
            },
            token_program,
            transfer_hook_accounts,
        )
    };

    let ix_data = presale::instruction::CreatorWithdraw {
        remaining_accounts_info,
    }
    .data();

    let mut accounts = presale::accounts::CreatorWithdrawCtx {
        presale,
        owner: owner_pubkey,
        event_authority: derive_event_authority(&presale::ID),
        token_program,
        program: presale::ID,
        owner_token,
        presale_authority: presale::presale_authority::ID,
        memo_program: anchor_spl::memo::ID,
    }
    .to_account_metas(None);

    accounts.push(AccountMeta {
        pubkey: token_vault,
        is_signer: false,
        is_writable: true,
    });
    accounts.push(AccountMeta {
        pubkey: mint,
        is_signer: false,
        is_writable: false,
    });
    accounts.extend(transfer_hook_accounts);

    let ix = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    process_transaction(
        lite_svm,
        &[create_owner_token_ix, ix],
        Some(&owner_pubkey),
        &[&owner],
    )
    .unwrap();
}
