use std::rc::Rc;

use crate::helpers::*;
use anchor_client::solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
};
use anchor_lang::{prelude::AccountMeta, *};
use litesvm::LiteSVM;
use mpl_token_metadata::accounts::Metadata;
use presale::{LockedVestingArgs, PresaleArgs, TokenInfoArgs, TokenomicArgs};

pub struct HandleInitializePresaleArgs {
    pub base_mint: Rc<Keypair>,
    pub quote_mint: Pubkey,
    pub token_info: TokenInfoArgs,
    pub tokenomic: TokenomicArgs,
    pub presale_params: PresaleArgs,
    pub locked_vesting_params: Option<LockedVestingArgs>,
    pub creator: Pubkey,
    pub payer: Rc<Keypair>,
    pub remaining_accounts: Vec<AccountMeta>,
}

pub fn handle_initialize_presale(lite_svm: &mut LiteSVM, args: HandleInitializePresaleArgs) {
    let HandleInitializePresaleArgs {
        base_mint,
        quote_mint,
        token_info,
        tokenomic,
        presale_params,
        locked_vesting_params,
        creator,
        payer,
        remaining_accounts,
    } = args;

    let base_mint_pubkey = base_mint.pubkey();

    let ix_data = presale::instruction::InitializePresale {
        params: presale::InitializePresaleArgs {
            token_info,
            tokenomic,
            presale_params,
            locked_vesting_params,
        },
    }
    .data();

    let presale = derive_presale(&base_mint_pubkey, &quote_mint, &presale::ID);
    let presale_vault = derive_presale_vault(&presale, &presale::ID);
    let quote_vault = derive_quote_vault(&presale, &presale::ID);
    let mint_metadata = Metadata::find_pda(&base_mint_pubkey).0;
    let event_authority = derive_event_authority(&presale::ID);

    let mut accounts = presale::accounts::InitializePresaleCtx {
        presale,
        mint: base_mint_pubkey,
        mint_metadata,
        metadata_program: mpl_token_metadata::ID,
        presale_authority: presale::presale_authority::ID,
        quote_token_mint: quote_mint,
        presale_vault,
        quote_token_vault: quote_vault,
        creator,
        payer: payer.pubkey(),
        token_program: anchor_spl::token::spl_token::ID,
        system_program: anchor_lang::solana_program::system_program::ID,
        event_authority,
        program: presale::ID,
    }
    .to_account_metas(None);

    accounts.extend_from_slice(&remaining_accounts);

    let instruction = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    process_transaction(
        lite_svm,
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer, &base_mint],
    );
}

pub fn handle_initialize_presale_token_2022(
    lite_svm: &mut LiteSVM,
    args: HandleInitializePresaleArgs,
) {
    let HandleInitializePresaleArgs {
        base_mint,
        quote_mint,
        token_info,
        tokenomic,
        presale_params,
        locked_vesting_params,
        creator,
        payer,
        remaining_accounts,
    } = args;

    let base_mint_pubkey = base_mint.pubkey();

    let ix_data = presale::instruction::InitializePresaleToken2022 {
        params: presale::InitializePresaleArgs {
            token_info,
            tokenomic,
            presale_params,
            locked_vesting_params,
        },
    }
    .data();

    let presale = derive_presale(&base_mint_pubkey, &quote_mint, &presale::ID);
    let presale_vault = derive_presale_vault(&presale, &presale::ID);
    let quote_vault = derive_quote_vault(&presale, &presale::ID);
    let event_authority = derive_event_authority(&presale::ID);

    let mut accounts = presale::accounts::InitializePresaleToken2022Ctx {
        presale,
        mint: base_mint_pubkey,
        presale_authority: presale::presale_authority::ID,
        quote_token_mint: quote_mint,
        presale_vault,
        quote_token_vault: quote_vault,
        creator,
        payer: payer.pubkey(),
        token_program: anchor_spl::token_2022::spl_token_2022::ID,
        system_program: anchor_lang::solana_program::system_program::ID,
        event_authority,
        program: presale::ID,
    }
    .to_account_metas(None);

    accounts.extend_from_slice(&remaining_accounts);

    let instruction = Instruction {
        program_id: presale::ID,
        accounts,
        data: ix_data,
    };

    println!("presale {}", presale);
    println!("Account index 1 {}", &instruction.accounts[1].pubkey);

    process_transaction(
        lite_svm,
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer, &base_mint],
    );
}
