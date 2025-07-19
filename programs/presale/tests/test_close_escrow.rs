pub mod helpers;

use anchor_client::solana_sdk::{
    native_token::LAMPORTS_PER_SOL, signature::Keypair, signer::Signer,
};
use helpers::*;
use presale::Presale;
use std::rc::Rc;

#[test]
fn test_close_escrow() {
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

    let user_1 = Rc::new(Keypair::new());
    let funding_amount = LAMPORTS_PER_SOL * 3;
    transfer_sol(
        &mut lite_svm,
        Rc::clone(&user),
        user_1.pubkey(),
        funding_amount,
    );
    wrap_sol(
        &mut lite_svm,
        Rc::clone(&user_1),
        funding_amount - LAMPORTS_PER_SOL,
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    let amount_0 = presale_state.presale_maximum_cap / 2;
    let amount_1 = presale_state.presale_maximum_cap - amount_0;

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
            max_amount: amount_0,
        },
    );

    handle_escrow_deposit(
        &mut lite_svm,
        HandleEscrowDepositArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
            max_amount: amount_1,
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();

    warp_time(
        &mut lite_svm,
        presale_state.vesting_end_time - presale_state.vest_duration / 2,
    );

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
        },
    );

    warp_time(&mut lite_svm, presale_state.vesting_end_time);

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    handle_escrow_claim(
        &mut lite_svm,
        HandleEscrowClaimArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
        },
    );

    handle_close_escrow(
        &mut lite_svm,
        HandleCloseEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user),
        },
    );

    handle_close_escrow(
        &mut lite_svm,
        HandleCloseEscrowArgs {
            presale: presale_pubkey,
            owner: Rc::clone(&user_1),
        },
    );

    let presale_state: Presale = lite_svm
        .get_deserialized_zc_account(&presale_pubkey)
        .unwrap();
    println!("Presale state after claim: {:?}", presale_state);
}
