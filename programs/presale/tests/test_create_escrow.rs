pub mod helpers;

use anchor_client::solana_sdk::signer::Signer;
use helpers::*;
use std::rc::Rc;

#[test]
fn test_initialize_permissionless_escrow() {
    let SetupContext { mut lite_svm, user } = SetupContext::initialize();

    let HandleCreatePredefinedPermissionlessFixedPricePresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
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
