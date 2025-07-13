mod helpers;

use anchor_client::solana_sdk::{native_token::LAMPORTS_PER_SOL, signer::Signer};
use helpers::*;
use presale::Presale;
use std::rc::Rc;

#[test]
fn test_deposit() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();

    let HandleCreatePredefinedPermissionlessFixedPricePresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
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
