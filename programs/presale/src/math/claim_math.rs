use crate::*;

pub fn calculate_dripped_amount_for_user(
    vesting_start_time: u64,
    vest_duration: u64,
    current_timestamp: u64,
    total_sold_token: u64,
    user_deposit: u64,
    total_deposit: u64,
) -> Result<u128> {
    let elapsed_seconds = current_timestamp
        .safe_sub(vesting_start_time)?
        .min(vest_duration);

    let dripped_total_sold_token = u128::from(total_sold_token)
        .safe_mul(elapsed_seconds.into())?
        .safe_div(vest_duration.into())?;

    let dripped_user_token = dripped_total_sold_token
        .safe_mul(user_deposit.into())?
        .safe_div(total_deposit.into())?;

    Ok(dripped_user_token)
}
