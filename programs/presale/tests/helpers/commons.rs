use std::rc::Rc;

use anchor_client::solana_client::rpc_response::RpcKeyedAccount;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::message::{Message, VersionedMessage};
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::transaction::VersionedTransaction;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use litesvm::LiteSVM;
use presale::TokenInfoArgs;

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

        load_programs(&mut svm);
        load_accounts(&mut svm);

        let user = Keypair::new();
        let user_address = user.pubkey();
        svm.airdrop(&user_address, 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to user");

        let rc_user = Rc::new(user);

        Self {
            lite_svm: svm,
            user: rc_user,
        }
    }
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

fn load_accounts(svm: &mut LiteSVM) {
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
            svm.set_account(account_pubkey, account).unwrap();
            println!(
                "Added account: {} with pubkey: {}",
                account_pubkey, account_pubkey
            );
        }
    }
}

pub fn create_token_info() -> TokenInfoArgs {
    TokenInfoArgs {
        decimals: 6,
        name: "Test Token".into(),
        symbol: "TT".into(),
        uri: "https://example.com/token/tt".into(),
    }
}

pub fn process_transaction(
    lite_svm: &mut LiteSVM,
    instructions: &[Instruction],
    payer: Option<&Pubkey>,
    signers: &[&Keypair],
) {
    let blockhash = lite_svm.latest_blockhash();
    let msg = Message::new_with_blockhash(instructions, payer, &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();

    let sim_res = lite_svm.simulate_transaction(tx.clone());
    if let Err(e) = sim_res {
        panic!("Simulation failed: {:?}", e);
    }

    lite_svm.send_transaction(tx).unwrap();
}
