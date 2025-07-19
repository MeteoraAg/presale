pub mod helpers;

use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use helpers::*;
use presale::Presale;
use std::rc::Rc;

#[test]
fn test_withdraw_remaining_quote() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();

    let token_decimals = 6;
    let base_mint = Rc::new(Keypair::new());

    create_token(CreateTokenArgs {
        lite_svm: &mut lite_svm,
        mint: Rc::clone(&base_mint),
        mint_authority: Rc::clone(&user),
        payer: Rc::clone(&user),
        decimals: token_decimals,
    });

    let HandleCreatePredefinedPermissionlessFixedPricePresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            base_mint.pubkey(),
            anchor_spl::token::spl_token::native_mint::ID,
            Rc::clone(&user),
        );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: 1000,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.presale_end_time + 1);

    handle_escrow_withdraw_remaining_quote(
        &mut lite_svm,
        HandleEscrowWithdrawRemainingQuoteArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();
    println!("Presale state after withdraw: {:?}", presale_state);
}
