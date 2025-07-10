use crate::*;

pub enum Rounding {
    Up,
    Down,
}

pub fn calculate_fee_amount(amount: u64, fee_bps: u16, rounding: Rounding) -> Result<u64> {
    match rounding {
        Rounding::Up => Ok(amount
            .checked_mul(fee_bps as u64)
            .unwrap()
            .checked_add(9999)
            .unwrap()
            .checked_div(10_000)
            .unwrap()),
        Rounding::Down => Ok(amount
            .checked_mul(fee_bps as u64)
            .unwrap()
            .checked_div(10_000)
            .unwrap()),
    }
}
