pub mod helpers;

use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use helpers::*;
use presale::Escrow;
use std::rc::Rc;

#[test]
fn test_withdraw_fixed_price_presale() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();
    let user_pubkey = user.pubkey();

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
            max_amount: 1_000_000,
        },
    );

    let escrow = derive_escrow(presale_pubkey, user_pubkey, &presale::ID);
    let escrow_state = lite_svm
        .get_deserialized_zc_account::<Escrow>(&escrow)
        .unwrap();

    handle_escrow_withdraw(
        &mut lite_svm,
        HandleEscrowWithdrawArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            amount: escrow_state.total_deposit,
        },
    );
}
