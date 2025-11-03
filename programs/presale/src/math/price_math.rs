use crate::*;

// Calculate min quote amount needed to purchase at least 1 base lamport. If price < 1 quote token, min quote amount will be > 1
pub fn calculate_min_quote_amount_for_base_lamport(q_price: u128) -> Result<u64> {
    let min_quote_amount = q_price.div_ceil(SCALE_MULTIPLIER);
    Ok(min_quote_amount.safe_cast()?)
}

pub fn calculate_token_bought(q_price: u128, amount: u64) -> Result<u128> {
    let q_amount = u128::from(amount).safe_shl(SCALE_OFFSET)?;
    let token_bought = q_amount.safe_div(q_price)?;

    Ok(token_bought)
}
