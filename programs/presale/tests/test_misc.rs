use presale::LockedVestingArgs;

use crate::helpers::calculate_q_price_from_ui_price;

pub mod helpers;

#[test]
fn test_calculate_q_price_from_ui_price() {
    let ui_price = 0.01;
    let base_decimals = 6;
    let quote_decimals = 9;

    let q_price = calculate_q_price_from_ui_price(ui_price, base_decimals, quote_decimals);

    let ui_amount_to_buy_token = 1.0;
    let ui_token_bought = ui_amount_to_buy_token / ui_price;

    println!("UI amount token bought: {}", ui_token_bought);

    let amount_to_buy_token =
        (ui_amount_to_buy_token * 10f64.powi(i32::from(quote_decimals))) as u64;

    let token_bought = u128::from(amount_to_buy_token).checked_shl(64).unwrap() / q_price;

    println!("Token bought: {}", token_bought);

    assert_eq!(
        token_bought,
        (ui_token_bought * 10f64.powi(i32::from(base_decimals))) as u128
    );
}

#[test]
fn test_calculate_presale_maximum_cap_from_usd_raise_target() {
    let quote_token_price = 183.33;
    let quote_token_decimals = 9;
    let usd_raise_target = 100_000_000.0f64; // 100 million

    let presale_maximum_cap =
        (usd_raise_target / quote_token_price * 10.0f64.powi(quote_token_decimals)) as u64;

    println!("Presale maximum cap: {}", presale_maximum_cap);
}

#[test]
fn test_calculate_q_price_from_market_cap_and_token_supply() {
    let quote_token_price = 183.33;
    let quote_token_decimals = 9;
    let base_token_decimals = 6;

    let market_cap = 1_000_000_000.0f64; // 1 billion
    let token_supply = 100_000_000.0f64; // 100 million

    // market_cap = token_price * quote_token_price * token_supply
    let token_price = market_cap / (quote_token_price * token_supply);
    println!("Token price: {}", token_price);

    let lamport_price = token_price * 10f64.powi(quote_token_decimals - base_token_decimals);
    println!("Lamport price: {}", lamport_price);

    let q_price = (lamport_price * 2.0f64.powi(64)) as u128;
    println!("Q price: {}", q_price);
}

#[test]
fn test_validate_locked_vesting_args() {
    let mut locked_vesting_args = LockedVestingArgs::default();
    let presale_end_time = 1_700_000_000;

    // All token is immediately released
    locked_vesting_args.immediately_release_bps = 10_000;
    locked_vesting_args.immediate_release_timestamp = presale_end_time + 1;

    // All token must immediately released upon presale end
    assert!(locked_vesting_args.validate(presale_end_time).is_err());
    locked_vesting_args.immediate_release_timestamp = presale_end_time;
    assert!(locked_vesting_args.validate(presale_end_time).is_ok());

    // Portion of token is released within presale end time and vesting end time
    let mut locked_vesting_args = LockedVestingArgs::default();
    locked_vesting_args.immediately_release_bps = 5000;
    locked_vesting_args.lock_duration = 60;
    locked_vesting_args.vest_duration = 60;
    locked_vesting_args.immediate_release_timestamp = presale_end_time - 1;
    assert!(locked_vesting_args.validate(presale_end_time).is_err());

    locked_vesting_args.immediate_release_timestamp = presale_end_time;
    assert!(locked_vesting_args.validate(presale_end_time).is_ok());

    locked_vesting_args.immediate_release_timestamp = presale_end_time + 120;
    assert!(locked_vesting_args.validate(presale_end_time).is_ok());

    locked_vesting_args.immediate_release_timestamp = presale_end_time + 121;
    assert!(locked_vesting_args.validate(presale_end_time).is_err());

    // All token is vested
    let mut locked_vesting_args = LockedVestingArgs::default();
    locked_vesting_args.immediately_release_bps = 0;
    locked_vesting_args.lock_duration = 60;
    locked_vesting_args.vest_duration = 60;
    locked_vesting_args.immediate_release_timestamp = presale_end_time + 30;

    // Immediate release timestamp must be presale end time
    assert!(locked_vesting_args.validate(presale_end_time).is_err());
    locked_vesting_args.immediate_release_timestamp = presale_end_time;
    assert!(locked_vesting_args.validate(presale_end_time).is_ok());
}
