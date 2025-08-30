use crate::*;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::MAX_FEE_BASIS_POINTS;

pub enum Rounding {
    Up,
    Down,
}

pub struct DepositFeeIncludedCalculation {
    pub fee: u64,
    pub amount_included_fee: u64,
}

pub fn calculate_deposit_fee_included_amount(
    deposit_amount: u64,
    fee_bps: u16,
    rounding: Rounding,
) -> Result<DepositFeeIncludedCalculation> {
    let denominator = u128::from(MAX_FEE_BASIS_POINTS.safe_sub(fee_bps)?);
    let adjust_deposit_amount = u128::from(deposit_amount).safe_mul(MAX_FEE_BASIS_POINTS.into())?;

    let adjusted_deposit_amount = match rounding {
        Rounding::Up => adjust_deposit_amount.safe_add(denominator.safe_sub(1)?)?,
        Rounding::Down => adjust_deposit_amount,
    };

    // Denominator always > 0
    let fee_included_deposit_amount = adjusted_deposit_amount.safe_div(denominator)?;
    let fee = fee_included_deposit_amount.safe_sub(u128::from(deposit_amount))?;

    Ok(DepositFeeIncludedCalculation {
        fee: fee.safe_cast()?,
        amount_included_fee: fee_included_deposit_amount.safe_cast()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_deposit_fee_included_amount_round_up() {
        let deposit_amount = 100_000_000;
        let fee_bps = 500; // 5%

        let DepositFeeIncludedCalculation {
            fee,
            amount_included_fee,
        } = calculate_deposit_fee_included_amount(deposit_amount, fee_bps, Rounding::Up).unwrap();

        let computed_fee = (u128::from(amount_included_fee) * u128::from(fee_bps))
            .div_ceil(u128::from(MAX_FEE_BASIS_POINTS));

        assert_eq!(u128::from(fee), computed_fee);
    }

    #[test]
    fn test_calculate_deposit_fee_included_amount_round_down() {
        let deposit_amount = 100_000_000;
        let fee_bps = 500; // 5%

        let DepositFeeIncludedCalculation {
            fee,
            amount_included_fee,
        } = calculate_deposit_fee_included_amount(deposit_amount, fee_bps, Rounding::Down).unwrap();

        let computed_fee = u128::from(amount_included_fee) * u128::from(fee_bps)
            / u128::from(MAX_FEE_BASIS_POINTS);

        assert_eq!(u128::from(fee), computed_fee);
    }
}
