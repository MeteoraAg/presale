pub mod helpers;

use anchor_client::solana_sdk::signer::Signer;
use helpers::*;
use presale::Presale;
use std::rc::Rc;

#[test]
fn test_unsold_token_action() {
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
            max_amount: 1_000_000,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(&mut lite_svm, presale_state.vesting_start_time);

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    handle_perform_unsold_token_action(
        &mut lite_svm,
        HandlePerformUnsoldTokenActionArgs {
            presale: presale_pubkey,
            creator: Rc::clone(&user),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();
    println!("Presale state after withdraw: {:?}", presale_state);
}
