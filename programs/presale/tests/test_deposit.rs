pub mod helpers;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use helpers::*;
use presale::Presale;
use std::rc::Rc;

#[test]
fn test_deposit() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();

    let token_decimals = 6;
    let mint = Rc::new(Keypair::new());

    create_token(CreateTokenArgs {
        lite_svm: &mut lite_svm,
        mint: Rc::clone(&mint),
        mint_authority: Rc::clone(&user),
        payer: Rc::clone(&user),
        decimals: token_decimals,
    });

    let HandleCreatePredefinedPermissionlessFixedPricePresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint.pubkey(),
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: 10 * LAMPORTS_PER_SOL,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();
    println!("Presale state after deposit: {:?}", presale_state);
}
