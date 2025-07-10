pub fn calculate_q_price_from_ui_price(price: f64, base_decimals: u8, quote_decimals: u8) -> u128 {
    let lamport_price = price * 10f64.powi(i32::from(quote_decimals) - i32::from(base_decimals));
    let q_price = lamport_price * 2.0f64.powi(64);
    q_price as u128
}
