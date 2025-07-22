pub mod helpers;

use helpers::*;
use std::rc::Rc;

#[test]
fn test_initialize_permissionless_escrow() {
    let mut setup_context = SetupContext::initialize();
    let mint = setup_context.setup_mint(
        DEFAULT_BASE_TOKEN_DECIMALS,
        1_000_000_000 * 10u64.pow(DEFAULT_BASE_TOKEN_DECIMALS.into()),
    );
    let SetupContext { mut lite_svm, user } = setup_context;

    let HandleCreatePredefinedPermissionlessFixedPricePresaleResponse { presale_pubkey, .. } =
        handle_create_predefined_permissionless_fixed_price_presale(
            &mut lite_svm,
            mint,
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
