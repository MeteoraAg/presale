use crate::*;

pub fn calculate_deposit_fee_included_amount(amount: u64, fee_bps: u16) -> Result<u64> {
    let denominator = 10_000u64.checked_sub(fee_bps as u64).unwrap();
    let include_fee_amount = amount
        .checked_mul(10_000)
        .unwrap()
        .checked_add(denominator - 1)
        .unwrap()
        .checked_div(denominator)
        .unwrap();

    Ok(include_fee_amount)
}

pub fn calculate_deposit_fee_included_amount_with_max_cap(
    amount: u64,
    fee_bps: u16,
    max_fee: u64,
) -> Result<u64> {
    let included_fee_amount = calculate_deposit_fee_included_amount(amount, fee_bps)?;
    let fee_amount = included_fee_amount.checked_sub(amount).unwrap();
    let capped_fee_amount = fee_amount.min(max_fee);

    let included_fee_amount = amount.checked_add(capped_fee_amount).unwrap();
    Ok(included_fee_amount)
}

pub fn calculate_dripped_amount_for_user(
    vesting_start_time: u64,
    vest_duration: u64,
    current_timestamp: u64,
    total_sold_token: u64,
    user_deposit: u64,
    total_deposit: u64,
) -> Result<u128> {
    let elapsed_seconds = current_timestamp
        .checked_sub(vesting_start_time)
        .unwrap()
        .min(vest_duration);

    let dripped_total_sold_token = u128::from(total_sold_token)
        .checked_mul(elapsed_seconds.into())
        .unwrap()
        .checked_div(vest_duration.into())
        .unwrap();

    let dripped_user_token = dripped_total_sold_token
        .checked_mul(user_deposit.into())
        .unwrap()
        .checked_div(total_deposit.into())
        .unwrap();

    Ok(dripped_user_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calculate_deposit_fee_amount(amount: u64, fee_bps: u16) -> u64 {
        amount
            .checked_mul(fee_bps as u64)
            .unwrap()
            .checked_add(9999)
            .unwrap()
            .checked_div(10_000)
            .unwrap()
    }

    fn calculate_deposit_fee_excluded_amount(amount: u64, fee_bps: u16) -> u64 {
        amount
            .checked_sub(calculate_deposit_fee_amount(amount, fee_bps))
            .unwrap()
    }

    #[test]
    fn test_calculate_deposit_fee_included_amount_reciprocal() {
        let amount = 100_000;
        let fee_bps = 100;

        let included_amount = calculate_deposit_fee_included_amount(amount, fee_bps).unwrap();
        let excluded_amount = calculate_deposit_fee_excluded_amount(included_amount, fee_bps);

        assert_eq!(excluded_amount, amount);
    }
}
