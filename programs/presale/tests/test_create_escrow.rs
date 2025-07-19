pub mod helpers;

use anchor_client::solana_sdk::{signature::Keypair, signer::Signer};
use helpers::*;
use std::rc::Rc;

#[test]
fn test_initialize_permissionless_escrow() {
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

    handle_create_permissionless_escrow(
        &mut lite_svm,
        HandleCreatePermissionlessEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );
}
