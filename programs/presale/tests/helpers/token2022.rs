use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::system_instruction::{create_account, transfer};
use anchor_lang::prelude::{AccountMeta, Rent};
use anchor_spl::token_2022::spl_token_2022::extension::ExtensionType;
use anchor_spl::token_2022::spl_token_2022::instruction::{initialize_mint, transfer_checked};
use anchor_spl::token_2022::spl_token_2022::state::Mint;
use anchor_spl::token_interface::spl_pod::optional_keys::OptionalNonZeroPubkey;
use anchor_spl::token_interface::spl_pod::slice::PodSlice;
use anchor_spl::token_interface::spl_token_metadata_interface;
use litesvm::LiteSVM;
use spl_discriminator::SplDiscriminate;
use spl_tlv_account_resolution::account::ExtraAccountMeta;
use spl_transfer_hook_interface::get_extra_account_metas_address;
use spl_transfer_hook_interface::instruction::{execute, ExecuteInstruction};
use spl_type_length_value::state::{TlvState, TlvStateBorrowed};

use crate::*;

pub struct ExtensionTypeWithInstructions {
    pub extension_type: ExtensionType,
    pub instructions: Vec<Instruction>,
    pub before_init_mint_ix: bool,
}

pub fn get_token_metadata_extension_type_with_instructions(
    mint_pubkey: Pubkey,
    mint_authority_pubkey: Pubkey,
) -> Vec<ExtensionTypeWithInstructions> {
    let mut instructions = vec![];

    let initialize_token_metadata_pointer_ix = anchor_spl::token_2022::spl_token_2022::extension::metadata_pointer::instruction::initialize(
        &anchor_spl::token_2022::spl_token_2022::ID,
        &mint_pubkey,
        Some(mint_authority_pubkey),
        Some(mint_pubkey),
    )
    .unwrap();

    instructions.push(ExtensionTypeWithInstructions {
        extension_type: ExtensionType::MetadataPointer,
        instructions: vec![initialize_token_metadata_pointer_ix],
        before_init_mint_ix: true,
    });

    let initialize_token_metadata_ix = spl_token_metadata_interface::instruction::initialize(
        &anchor_spl::token_2022::spl_token_2022::ID,
        &mint_pubkey,
        &mint_authority_pubkey,
        &mint_pubkey,
        &mint_authority_pubkey,
        "TOKEN NAME".to_string(),
        "TOKEN".to_string(),
        "https://token-uri.com".to_string(),
    );

    let revoke_update_authority_ix = spl_token_metadata_interface::instruction::update_authority(
        &anchor_spl::token_2022::spl_token_2022::ID,
        &mint_pubkey,
        &mint_authority_pubkey,
        OptionalNonZeroPubkey::try_from(Option::<Pubkey>::None).unwrap(),
    );

    instructions.push(ExtensionTypeWithInstructions {
        extension_type: ExtensionType::TokenMetadata,
        instructions: vec![initialize_token_metadata_ix, revoke_update_authority_ix],
        before_init_mint_ix: false,
    });

    instructions
}

pub struct CreateToken2022Args<'a> {
    pub lite_svm: &'a mut LiteSVM,
    pub mint: Rc<Keypair>,
    pub mint_authority: Rc<Keypair>,
    pub payer: Rc<Keypair>,
    pub decimals: u8,
    pub extension_type_with_instructions: Vec<ExtensionTypeWithInstructions>,
}

pub fn create_token_2022(args: CreateToken2022Args) {
    let CreateToken2022Args {
        lite_svm,
        mint,
        mint_authority,
        payer,
        decimals,
        extension_type_with_instructions,
    } = args;

    let mint_pubkey = mint.pubkey();
    let mint_authority_pubkey = mint_authority.pubkey();
    let payer_pubkey = payer.pubkey();

    let rent = lite_svm.get_sysvar::<Rent>();

    let before_init_mint_ix_extensions_type = extension_type_with_instructions
        .iter()
        .filter(|ext| ext.before_init_mint_ix)
        .map(|ext| ext.extension_type)
        .collect::<Vec<_>>();

    let space =
        ExtensionType::try_calculate_account_len::<Mint>(&before_init_mint_ix_extensions_type)
            .unwrap();

    let lamports = rent.minimum_balance(space);

    let create_account_ix = create_account(
        &payer_pubkey,
        &mint_pubkey,
        lamports,
        space as u64,
        &anchor_spl::token_2022::spl_token_2022::ID,
    );

    let initialize_mint_ix = initialize_mint(
        &anchor_spl::token_2022::spl_token_2022::ID,
        &mint_pubkey,
        &mint_authority_pubkey,
        None,
        decimals,
    )
    .expect("Failed to create initialize_mint instruction");

    let mut instructions = vec![create_account_ix];

    for ix in extension_type_with_instructions
        .iter()
        .filter(|ext| ext.before_init_mint_ix)
        .map(|ext| ext.instructions.clone())
    {
        instructions.extend_from_slice(&ix);
    }

    instructions.push(initialize_mint_ix);

    for ix in extension_type_with_instructions
        .iter()
        .filter(|ext| !ext.before_init_mint_ix)
        .map(|ext| ext.instructions.clone())
    {
        instructions.extend_from_slice(&ix);
    }

    // TODO: Should calculate variable length extension types require how many extra lamports
    instructions.push(transfer(&payer_pubkey, &mint_pubkey, 10_000_000));

    process_transaction(
        lite_svm,
        &instructions,
        Some(&payer_pubkey),
        &[&payer, &mint],
    )
    .unwrap();
}

pub struct MintTokenArgs<'a> {
    pub lite_svm: &'a mut LiteSVM,
    pub mint: Pubkey,
    pub amount: u64,
    pub destination: Pubkey,
    pub mint_authority: Rc<Keypair>,
}

pub fn get_extra_account_metas_for_transfer_hook(
    program_id: &Pubkey,
    source_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    lite_svm: &LiteSVM,
) -> Vec<AccountMeta> {
    if program_id != &anchor_spl::token_2022::spl_token_2022::ID {
        return vec![];
    }

    let mut dummy_transfer_ix = transfer_checked(
        program_id,
        source_pubkey,
        mint_pubkey,
        destination_pubkey,
        authority_pubkey,
        &[],
        0,
        0,
    )
    .unwrap();

    add_extra_account_metas_for_execute(
        &mut dummy_transfer_ix,
        program_id,
        source_pubkey,
        mint_pubkey,
        destination_pubkey,
        authority_pubkey,
        0,
        lite_svm,
    );

    let extra_account_metas_slice = dummy_transfer_ix
        .accounts
        .iter()
        .skip(4)
        .map(|acc| acc.clone())
        .collect::<Vec<_>>();

    extra_account_metas_slice
}

fn add_extra_account_metas_for_execute(
    instruction: &mut Instruction,
    program_id: &Pubkey,
    source_pubkey: &Pubkey,
    mint_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    amount: u64,
    lite_svm: &LiteSVM,
) {
    let validate_state_pubkey = get_extra_account_metas_address(mint_pubkey, program_id);
    let validate_state_data = lite_svm
        .get_account(&validate_state_pubkey)
        .expect("Failed to get validate state account")
        .data;

    // Check to make sure the provided keys are in the instruction
    if [
        source_pubkey,
        mint_pubkey,
        destination_pubkey,
        authority_pubkey,
    ]
    .iter()
    .any(|&key| !instruction.accounts.iter().any(|meta| meta.pubkey == *key))
    {
        panic!("Instruction does not contain all required accounts");
    }

    let mut execute_instruction = execute(
        program_id,
        source_pubkey,
        mint_pubkey,
        destination_pubkey,
        authority_pubkey,
        amount,
    );

    execute_instruction
        .accounts
        .push(AccountMeta::new_readonly(validate_state_pubkey, false));

    add_to_instruction::<ExecuteInstruction>(
        &mut execute_instruction,
        lite_svm,
        &validate_state_data,
    );

    // Add only the extra accounts resolved from the validation state
    instruction
        .accounts
        .extend_from_slice(&execute_instruction.accounts[5..]);

    // Add the program id and validation state account
    instruction
        .accounts
        .push(AccountMeta::new_readonly(*program_id, false));
    instruction
        .accounts
        .push(AccountMeta::new_readonly(validate_state_pubkey, false));
}

fn add_to_instruction<T: SplDiscriminate>(
    instruction: &mut Instruction,
    lite_svm: &LiteSVM,
    data: &[u8],
) {
    let state = TlvStateBorrowed::unpack(data).unwrap();
    let bytes = state.get_first_bytes::<T>().unwrap();
    let extra_account_metas = PodSlice::<ExtraAccountMeta>::unpack(bytes).unwrap();

    // Fetch account data for each of the instruction accounts
    let mut account_key_datas = vec![];
    for meta in instruction.accounts.iter() {
        let account_data = lite_svm.get_account(&meta.pubkey).map(|acc| acc.data);
        account_key_datas.push((meta.pubkey, account_data));
    }

    for extra_meta in extra_account_metas.data().iter() {
        let mut meta = extra_meta
            .resolve(&instruction.data, &instruction.program_id, |usize| {
                account_key_datas
                    .get(usize)
                    .map(|(pubkey, opt_data)| (pubkey, opt_data.as_ref().map(|x| x.as_slice())))
            })
            .unwrap();
        de_escalate_account_meta(&mut meta, &instruction.accounts);

        // Fetch account data for the new account
        account_key_datas.push((
            meta.pubkey,
            lite_svm.get_account(&meta.pubkey).map(|acc| acc.data),
        ));
        instruction.accounts.push(meta);
    }
}

/// De-escalate an account meta if necessary
fn de_escalate_account_meta(account_meta: &mut AccountMeta, account_metas: &[AccountMeta]) {
    // This is a little tricky to read, but the idea is to see if
    // this account is marked as writable or signer anywhere in
    // the instruction at the start. If so, DON'T escalate it to
    // be a writer or signer in the CPI
    let maybe_highest_privileges = account_metas
        .iter()
        .filter(|&x| x.pubkey == account_meta.pubkey)
        .map(|x| (x.is_signer, x.is_writable))
        .reduce(|acc, x| (acc.0 || x.0, acc.1 || x.1));
    // If `Some`, then the account was found somewhere in the instruction
    if let Some((is_signer, is_writable)) = maybe_highest_privileges {
        if !is_signer && is_signer != account_meta.is_signer {
            // Existing account is *NOT* a signer already, but the CPI
            // wants it to be, so de-escalate to not be a signer
            account_meta.is_signer = false;
        }
        if !is_writable && is_writable != account_meta.is_writable {
            // Existing account is *NOT* writable already, but the CPI
            // wants it to be, so de-escalate to not be writable
            account_meta.is_writable = false;
        }
    }
}
