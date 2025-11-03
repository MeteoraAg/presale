use std::rc::Rc;
use std::time::SystemTime;

use anchor_client::solana_client::rpc_response::RpcKeyedAccount;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::message::{Message, VersionedMessage};
use anchor_client::solana_sdk::program_option::COption;
use anchor_client::solana_sdk::program_pack::Pack;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::transaction::VersionedTransaction;
use anchor_lang::prelude::{Clock, Rent};
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account_idempotent;
use anchor_spl::associated_token::{
    get_associated_token_address, get_associated_token_address_with_program_id,
};
use anchor_spl::token::spl_token::state::AccountState;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::TransferHook;
use anchor_spl::token_2022::spl_token_2022::extension::{
    BaseStateWithExtensions, StateWithExtensions,
};
use anchor_spl::token_2022::spl_token_2022::instruction::{mint_to, transfer_checked};
use anchor_spl::token_2022::spl_token_2022::state::Mint;
use litesvm::types::{FailedTransactionMetadata, SimulatedTransactionInfo};
use litesvm::LiteSVM;
use presale::Presale;

use crate::helpers::{
    add_extra_account_metas_for_execute, create_token_2022_ix, create_token_ix,
    get_token_metadata_extension_type_with_instructions,
    get_transfer_fee_extension_type_with_instructions,
    get_transfer_hook_extension_type_with_instructions, CreateToken2022Args, CreateTokenArgs,
    ExtensionTypeWithInstructions, TRANSFER_HOOK_COUNTER_PROGRAM_ID,
};

const NATIVE_SOL_MINT: Pubkey =
    Pubkey::from_str_const("So11111111111111111111111111111111111111112");

pub const DEFAULT_BASE_TOKEN_DECIMALS: u8 = 6;
pub const DEFAULT_QUOTE_TOKEN_DECIMALS: u8 = 9;

pub fn decode_presale_mode_handler<T: bytemuck::Pod>(presale: &Presale) -> T {
    let slice = bytemuck::try_cast_slice::<u64, u8>(&presale.presale_mode_raw_data).unwrap();
    let handler = *bytemuck::try_from_bytes::<T>(slice).unwrap();
    handler
}

// TODO: Refactor this to allow setup user, and user struct can interact with the program
pub struct SetupContext {
    pub lite_svm: LiteSVM,
    pub user: Rc<Keypair>,
}

impl SetupContext {
    pub fn initialize() -> Self {
        let mut svm = LiteSVM::new()
            .with_sysvars()
            .with_lamports(10_000 * LAMPORTS_PER_SOL)
            .with_spl_programs()
            .with_sigverify(true)
            .with_blockhash_check(true);

        let user = Rc::new(Keypair::new());
        let user_address = user.pubkey();

        svm.airdrop(&user_address, 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to user");

        load_programs(&mut svm);
        load_accounts(&mut svm, Rc::clone(&user));

        adjust_clock_to_current_time(&mut svm);

        Self {
            lite_svm: svm,
            user,
        }
    }

    pub fn create_user(&mut self) -> Rc<Keypair> {
        let user = Rc::new(Keypair::new());
        let user_address = user.pubkey();

        self.lite_svm
            .airdrop(&user_address, 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to user");

        user
    }

    fn create_and_mint_token_ix(
        &mut self,
        token_decimals: u8,
        supply: u64,
        mint: Rc<Keypair>,
    ) -> Vec<Instruction> {
        let user_pubkey = self.user.pubkey();

        let instructions = create_token_ix(
            &mut self.lite_svm,
            CreateTokenArgs {
                mint: Rc::clone(&mint),
                mint_authority: Rc::clone(&self.user),
                payer: Rc::clone(&self.user),
                decimals: token_decimals,
            },
        );

        let user_ata = get_associated_token_address(&user_pubkey, &mint.pubkey());

        let create_user_ata_ix = create_associated_token_account_idempotent(
            &user_pubkey,
            &user_pubkey,
            &mint.pubkey(),
            &anchor_spl::token::ID,
        );

        let mint_ix = mint_to(
            &anchor_spl::token::ID,
            &mint.pubkey(),
            &user_ata,
            &user_pubkey,
            &[&user_pubkey],
            supply,
        )
        .unwrap();

        [instructions, vec![create_user_ata_ix, mint_ix]].concat()
    }

    pub fn setup_mint(&mut self, token_decimals: u8, supply: u64) -> Pubkey {
        let mint = Rc::new(Keypair::new());
        let user_pubkey = self.user.pubkey();

        let instructions = self.create_and_mint_token_ix(token_decimals, supply, Rc::clone(&mint));

        process_transaction(
            &mut self.lite_svm,
            &instructions,
            Some(&user_pubkey),
            &[&self.user, &mint],
        )
        .unwrap();

        mint.pubkey()
    }

    fn create_token_2022_and_mint_ix(
        &mut self,
        mint: Rc<Keypair>,
        token_decimals: u8,
        supply: u64,
        extension_type_with_instructions: Vec<ExtensionTypeWithInstructions>,
    ) -> Vec<Instruction> {
        let mint_pubkey = mint.pubkey();

        let create_token_ix = create_token_2022_ix(
            &mut self.lite_svm,
            CreateToken2022Args {
                mint: Rc::clone(&mint),
                mint_authority: Rc::clone(&self.user),
                payer: Rc::clone(&self.user),
                decimals: token_decimals,
                extension_type_with_instructions,
            },
        );

        let user_pubkey = self.user.pubkey();
        let user_ata = get_associated_token_address_with_program_id(
            &user_pubkey,
            &mint_pubkey,
            &anchor_spl::token_2022::ID,
        );

        let create_user_ata_ix = create_associated_token_account_idempotent(
            &user_pubkey,
            &user_pubkey,
            &mint_pubkey,
            &anchor_spl::token_2022::ID,
        );

        let mint_ix = mint_to(
            &anchor_spl::token_2022::ID,
            &mint.pubkey(),
            &user_ata,
            &user_pubkey,
            &[&user_pubkey],
            supply,
        )
        .unwrap();

        [create_token_ix, vec![create_user_ata_ix, mint_ix]].concat()
    }

    pub fn setup_token_2022_mint(&mut self, token_decimals: u8, supply: u64) -> Pubkey {
        let mint = Rc::new(Keypair::new());
        let mint_pubkey = mint.pubkey();

        let instructions = self.create_token_2022_and_mint_ix(
            Rc::clone(&mint),
            token_decimals,
            supply,
            get_token_metadata_extension_type_with_instructions(
                mint_pubkey,
                self.user.pubkey(),
                true,
            ),
        );

        process_transaction(
            &mut self.lite_svm,
            &instructions,
            Some(&self.user.pubkey()),
            &[&self.user, &mint],
        )
        .unwrap();

        mint.pubkey()
    }

    pub fn setup_token_2022_mint_with_transfer_hook_and_fee(
        &mut self,
        token_decimals: u8,
        supply: u64,
    ) -> Pubkey {
        let mint = Rc::new(Keypair::new());
        let mint_pubkey = mint.pubkey();

        let instructions = self.create_token_2022_and_mint_ix(
            Rc::clone(&mint),
            token_decimals,
            supply,
            [
                get_token_metadata_extension_type_with_instructions(
                    mint_pubkey,
                    self.user.pubkey(),
                    true,
                ),
                get_transfer_hook_extension_type_with_instructions(
                    mint_pubkey,
                    self.user.pubkey(),
                    TRANSFER_HOOK_COUNTER_PROGRAM_ID,
                ),
                get_transfer_fee_extension_type_with_instructions(
                    mint_pubkey,
                    self.user.pubkey(),
                    100,
                    1_000_000,
                ),
            ]
            .concat(),
        );

        process_transaction(
            &mut self.lite_svm,
            &instructions,
            Some(&self.user.pubkey()),
            &[&self.user, &mint],
        )
        .unwrap();

        mint.pubkey()
    }

    pub fn setup_token_2022_mint_with_transfer_hook(
        &mut self,
        token_decimals: u8,
        supply: u64,
    ) -> Pubkey {
        let mint = Rc::new(Keypair::new());
        let mint_pubkey = mint.pubkey();

        let instructions = self.create_token_2022_and_mint_ix(
            Rc::clone(&mint),
            token_decimals,
            supply,
            [
                get_token_metadata_extension_type_with_instructions(
                    mint_pubkey,
                    self.user.pubkey(),
                    true,
                ),
                get_transfer_hook_extension_type_with_instructions(
                    mint_pubkey,
                    self.user.pubkey(),
                    TRANSFER_HOOK_COUNTER_PROGRAM_ID,
                ),
            ]
            .concat(),
        );

        process_transaction(
            &mut self.lite_svm,
            &instructions,
            Some(&self.user.pubkey()),
            &[&self.user, &mint],
        )
        .unwrap();

        mint.pubkey()
    }

    pub fn setup_token_2022_mint_with_transfer_fee(
        &mut self,
        token_decimals: u8,
        supply: u64,
    ) -> Pubkey {
        let mint = Rc::new(Keypair::new());
        let mint_pubkey = mint.pubkey();

        let instructions = self.create_token_2022_and_mint_ix(
            Rc::clone(&mint),
            token_decimals,
            supply,
            [
                get_token_metadata_extension_type_with_instructions(
                    mint_pubkey,
                    self.user.pubkey(),
                    true,
                ),
                get_transfer_fee_extension_type_with_instructions(
                    mint_pubkey,
                    self.user.pubkey(),
                    100,
                    1_000_000,
                ),
            ]
            .concat(),
        );

        process_transaction(
            &mut self.lite_svm,
            &instructions,
            Some(&self.user.pubkey()),
            &[&self.user, &mint],
        )
        .unwrap();

        mint.pubkey()
    }

    pub fn setup_token_2022_mint_without_metadata(
        &mut self,
        token_decimals: u8,
        supply: u64,
    ) -> Pubkey {
        let mint = Rc::new(Keypair::new());

        let instructions =
            self.create_token_2022_and_mint_ix(Rc::clone(&mint), token_decimals, supply, vec![]);

        process_transaction(
            &mut self.lite_svm,
            &instructions,
            Some(&self.user.pubkey()),
            &[&self.user, &mint],
        )
        .unwrap();

        mint.pubkey()
    }

    pub fn setup_token_2022_mint_with_mutable_metadata(
        &mut self,
        token_decimals: u8,
        supply: u64,
    ) -> Pubkey {
        let mint = Rc::new(Keypair::new());
        let mint_pubkey = mint.pubkey();

        let instructions = self.create_token_2022_and_mint_ix(
            Rc::clone(&mint),
            token_decimals,
            supply,
            get_token_metadata_extension_type_with_instructions(
                mint_pubkey,
                self.user.pubkey(),
                false,
            ),
        );

        process_transaction(
            &mut self.lite_svm,
            &instructions,
            Some(&self.user.pubkey()),
            &[&self.user, &mint],
        )
        .unwrap();

        mint.pubkey()
    }
}

fn adjust_clock_to_current_time(lite_svm: &mut LiteSVM) {
    let current_timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut clock: Clock = lite_svm.get_sysvar();
    clock.unix_timestamp = current_timestamp as i64;
    lite_svm.set_sysvar(&clock);
}

fn load_programs(svm: &mut LiteSVM) {
    let program_path = format!(
        "{}/../../target/deploy/presale.so",
        env!("CARGO_MANIFEST_DIR")
    );
    println!("Loading program from: {}", program_path);
    let program_bytes = std::fs::read(program_path).expect("Failed to read program file");
    svm.add_program(presale::ID, &program_bytes);

    let other_program_path = format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"));
    println!("Loading other programs from: {}", other_program_path);

    let dir = std::fs::read_dir(other_program_path).expect("Failed to read fixtures directory");
    for entry in dir {
        let path = entry.expect("Failed to read entry").path();
        if path.extension().and_then(|s| s.to_str()) == Some("so") {
            let program_bytes = std::fs::read(&path).expect("Failed to read program file");
            let program_address = path
                .file_stem()
                .expect("Failed to get file stem")
                .to_str()
                .unwrap();
            let program_pubkey = Pubkey::from_str_const(program_address);
            svm.add_program(program_pubkey, &program_bytes);
            println!(
                "Added program: {} with pubkey: {}",
                program_address, program_pubkey
            );
        }
    }
}

fn create_user_token_account_when_is_mint_account(
    svm: &mut LiteSVM,
    user_keypair: Rc<Keypair>,
    account: &anchor_client::solana_sdk::account::Account,
    account_pubkey: Pubkey,
) {
    if account.owner != anchor_spl::token::ID {
        return;
    }

    let user = user_keypair.pubkey();

    let ata_pubkey =
        get_associated_token_address_with_program_id(&user, &NATIVE_SOL_MINT, &account.owner);

    if account_pubkey == NATIVE_SOL_MINT {
        let create_ata_ix = create_associated_token_account_idempotent(
            &user,
            &user,
            &NATIVE_SOL_MINT,
            &account.owner,
        );

        let transfer_ix = anchor_client::solana_sdk::system_instruction::transfer(
            &user,
            &ata_pubkey,
            100 * LAMPORTS_PER_SOL,
        );

        let sync_native_ix =
            anchor_spl::token::spl_token::instruction::sync_native(&account.owner, &ata_pubkey)
                .unwrap();

        process_transaction(
            svm,
            &[create_ata_ix, transfer_ix, sync_native_ix],
            Some(&user),
            &[&user_keypair],
        )
        .unwrap();

        return;
    }

    if let Ok(mint_account) = anchor_spl::token::spl_token::state::Mint::unpack(&account.data) {
        let decimals = mint_account.decimals;
        let token_account = anchor_spl::token::spl_token::state::Account {
            mint: account_pubkey,
            owner: user,
            amount: 100_000 * 10u64.pow(decimals as u32),
            delegate: COption::None,
            is_native: COption::None,
            state: AccountState::Initialized,
            delegated_amount: 0,
            close_authority: COption::None,
        };

        let mut data = [0u8; anchor_spl::token::spl_token::state::Account::LEN];
        let rent: Rent = svm.get_sysvar();
        let lamports = rent.minimum_balance(data.len());
        anchor_spl::token::spl_token::state::Account::pack(token_account, &mut data)
            .expect("Failed to pack token account");
        svm.set_account(
            ata_pubkey,
            anchor_client::solana_sdk::account::Account {
                lamports,
                data: data.to_vec(),
                owner: account.owner,
                executable: false,
                rent_epoch: 0,
            },
        )
        .expect("Failed to set user token account");
    }
}

fn load_accounts(svm: &mut LiteSVM, user_keypair: Rc<Keypair>) {
    let accounts_path = format!("{}/tests/fixtures/accounts", env!("CARGO_MANIFEST_DIR"));

    let accounts_dir = std::fs::read_dir(accounts_path).expect("Failed to read accounts directory");
    for entry in accounts_dir {
        let path = entry.expect("Failed to read entry").path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let account_data = std::fs::read_to_string(&path).expect("Failed to read account file");
            let rpc_account: RpcKeyedAccount =
                serde_json::from_str(&account_data).expect("Failed to deserialize account data");
            let account: anchor_client::solana_sdk::account::Account =
                rpc_account.account.decode().unwrap();
            let account_pubkey = Pubkey::from_str_const(&rpc_account.pubkey);

            svm.set_account(account_pubkey, account.clone()).unwrap();
            println!(
                "Added account: {} with pubkey: {}",
                account_pubkey, account_pubkey
            );

            create_user_token_account_when_is_mint_account(
                svm,
                Rc::clone(&user_keypair),
                &account,
                account_pubkey,
            );
        }
    }
}

pub fn process_transaction(
    lite_svm: &mut LiteSVM,
    instructions: &[Instruction],
    payer: Option<&Pubkey>,
    signers: &[&Keypair],
) -> Result<SimulatedTransactionInfo, FailedTransactionMetadata> {
    let blockhash = lite_svm.latest_blockhash();
    let msg = Message::new_with_blockhash(instructions, payer, &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();

    let sim_res = lite_svm.simulate_transaction(tx.clone())?;

    lite_svm.send_transaction(tx).unwrap();
    lite_svm.expire_blockhash();

    Ok(sim_res)
}

pub fn transfer_sol(lite_svm: &mut LiteSVM, user: Rc<Keypair>, destination: Pubkey, amount: u64) {
    let transfer_ix = anchor_client::solana_sdk::system_instruction::transfer(
        &user.pubkey(),
        &destination,
        amount,
    );

    process_transaction(lite_svm, &[transfer_ix], Some(&user.pubkey()), &[&user]).unwrap();
}

pub fn transfer_token(
    lite_svm: &mut LiteSVM,
    user: Rc<Keypair>,
    destination: Pubkey,
    mint: Pubkey,
    amount: u64,
) {
    let mint_account = lite_svm.get_account(&mint).unwrap();
    let mint_owner = mint_account.owner;

    let mint_state = StateWithExtensions::<Mint>::unpack(mint_account.data.as_slice()).unwrap();

    let create_ata_ix = create_associated_token_account_idempotent(
        &user.pubkey(),
        &destination,
        &mint,
        &mint_owner,
    );

    let from_ata = get_associated_token_address_with_program_id(&user.pubkey(), &mint, &mint_owner);
    let to_ata = get_associated_token_address_with_program_id(&destination, &mint, &mint_owner);

    let mut transfer_ix = transfer_checked(
        &mint_owner,
        &from_ata,
        &mint,
        &to_ata,
        &user.pubkey(),
        &[&user.pubkey()],
        amount,
        mint_state.base.decimals,
    )
    .unwrap();

    if let Some(transfer_hook_program_id) = mint_state
        .get_extension::<TransferHook>()
        .map(|ext| Option::<Pubkey>::from(ext.program_id))
        .ok()
        .flatten()
    {
        add_extra_account_metas_for_execute(
            &mut transfer_ix,
            &transfer_hook_program_id,
            &from_ata,
            &mint,
            &to_ata,
            &user.pubkey(),
            amount,
            lite_svm,
        );
    };

    process_transaction(
        lite_svm,
        &[create_ata_ix, transfer_ix],
        Some(&user.pubkey()),
        &[&user],
    )
    .unwrap();
}

pub fn wrap_sol(lite_svm: &mut LiteSVM, user: Rc<Keypair>, amount: u64) {
    let wsol_ata = get_associated_token_address(&user.pubkey(), &NATIVE_SOL_MINT);
    let create_ata_ix = create_associated_token_account_idempotent(
        &user.pubkey(),
        &user.pubkey(),
        &NATIVE_SOL_MINT,
        &anchor_spl::token::ID,
    );

    let mut instructions = vec![create_ata_ix];

    let transfer_ix =
        anchor_client::solana_sdk::system_instruction::transfer(&user.pubkey(), &wsol_ata, amount);
    instructions.push(transfer_ix);

    let sync_native_ix =
        anchor_spl::token::spl_token::instruction::sync_native(&anchor_spl::token::ID, &wsol_ata)
            .unwrap();
    instructions.push(sync_native_ix);

    process_transaction(lite_svm, &instructions, Some(&user.pubkey()), &[&user]).unwrap();
}

pub fn warp_time(lite_svm: &mut LiteSVM, timestamp: u64) {
    let mut clock: Clock = lite_svm.get_sysvar();
    clock.unix_timestamp = timestamp as i64;
    lite_svm.set_sysvar(&clock);
}

pub fn warp_to_presale_end(lite_svm: &mut LiteSVM, presale: &Presale) {
    let presale_end_time = presale.presale_end_time;
    warp_time(lite_svm, presale_end_time);
}
