use crate::*;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::MAX_FEE_BASIS_POINTS;

pub struct TokenReleaseResult {
    pub immediate_released_amount: u64,
    pub vested_amount: u64,
}

pub fn calculate_immediate_release_token(
    total_sold_token: u64,
    immediate_release_bps: u16,
) -> Result<TokenReleaseResult> {
    let immediate_released_amount = u128::from(total_sold_token)
        .safe_mul(immediate_release_bps.into())?
        .safe_div(MAX_FEE_BASIS_POINTS.into())?
        .safe_cast()?;

    let vested_amount = total_sold_token.safe_sub(immediate_released_amount)?;

    Ok(TokenReleaseResult {
        immediate_released_amount,
        vested_amount,
    })
}

pub fn calculate_immediate_release_token_for_user(
    release_amount: u64,
    user_deposit: u64,
    total_deposit: u64,
) -> Result<u64> {
    let user_immediate_release_token = u128::from(release_amount)
        .safe_mul(user_deposit.into())?
        .safe_div(total_deposit.into())?
        .safe_cast()?;

    Ok(user_immediate_release_token)
}

pub fn calculate_dripped_amount_for_user(
    vesting_start_time: u64,
    vest_duration: u64,
    current_timestamp: u64,
    vested_amount: u64,
    user_deposit: u64,
    total_deposit: u64,
) -> Result<u128> {
    if current_timestamp < vesting_start_time {
        return Ok(0);
    }

    let dripped_total_sold_token = if vest_duration == 0 {
        u128::from(vested_amount)
    } else {
        let elapsed_seconds = current_timestamp
            .safe_sub(vesting_start_time)?
            .min(vest_duration);

        u128::from(vested_amount)
            .safe_mul(elapsed_seconds.into())?
            .safe_div(vest_duration.into())?
    };

    let dripped_user_token = dripped_total_sold_token
        .safe_mul(user_deposit.into())?
        .safe_div(total_deposit.into())?;

    Ok(dripped_user_token)
}

pub fn calculate_cumulative_claimable_amount_for_user(
    immediate_release_bps: u16,
    total_sold_token: u64,
    vesting_start_time: u64,
    vest_duration: u64,
    current_timestamp: u64,
    user_deposit: u64,
    total_deposit: u64,
) -> Result<u64> {
    if user_deposit == 0 || total_deposit == 0 {
        return Ok(0);
    }

    let TokenReleaseResult {
        immediate_released_amount,
        vested_amount,
    } = calculate_immediate_release_token(total_sold_token, immediate_release_bps)?;

    let user_immediate_release_token = calculate_immediate_release_token_for_user(
        immediate_released_amount,
        user_deposit,
        total_deposit,
    )?;

    let user_dripped_token = calculate_dripped_amount_for_user(
        vesting_start_time,
        vest_duration,
        current_timestamp,
        vested_amount,
        user_deposit,
        total_deposit,
    )?
    .safe_cast()?;

    let cumulative_claimable_amount = user_immediate_release_token.safe_add(user_dripped_token)?;
    Ok(cumulative_claimable_amount)
}
