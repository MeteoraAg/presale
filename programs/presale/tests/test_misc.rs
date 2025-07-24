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
