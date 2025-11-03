use crate::PresaleModeHandler;
use crate::*;

fn calculate_quote_token_without_surplus(q_price: u128, amount: u64) -> Result<u64> {
    let base_token_amount = calculate_token_bought(q_price, amount)?;

    let quote_token_needed = base_token_amount
        .safe_mul(q_price)?
        .div_ceil(SCALE_MULTIPLIER);

    let quote_token_needed: u64 = quote_token_needed.safe_cast()?;

    // This should never happen
    require!(
        quote_token_needed <= amount,
        PresaleError::UndeterminedError
    );

    Ok(quote_token_needed)
}

#[zero_copy]
pub struct FixedPricePresaleHandler {
    pub q_price: u128,
    pub disable_withdraw: u8,
    pub disable_earlier_presale_end_once_cap_reached: u8,
    pub padding0: [u8; 6],
    pub padding1: [u64; 29],
}

impl FixedPricePresaleHandler {
    pub fn initialize<'a>(
        presale_mode_raw_data: &'a mut [u64; 32],
        q_price: u128,
        disable_withdraw: bool,
        disable_earlier_presale_end_once_cap_reached: bool,
    ) -> Result<()> {
        require!(q_price > 0, PresaleError::InvalidTokenPrice);

        let data = bytemuck::try_cast_slice_mut::<u64, u8>(presale_mode_raw_data)
            .map_err(|_| PresaleError::UndeterminedError)?;

        let fixed_price_presale_handler = bytemuck::try_from_bytes_mut::<Self>(data)
            .map_err(|_| PresaleError::UndeterminedError)?;

        fixed_price_presale_handler.disable_earlier_presale_end_once_cap_reached =
            disable_earlier_presale_end_once_cap_reached.into();
        fixed_price_presale_handler.disable_withdraw = disable_withdraw.into();
        fixed_price_presale_handler.q_price = q_price;

        Ok(())
    }

    pub fn is_withdraw_disabled(&self) -> bool {
        self.disable_withdraw == 1
    }

    pub fn is_earlier_presale_end_once_cap_reached_disabled(&self) -> bool {
        self.disable_earlier_presale_end_once_cap_reached == 1
    }
}

impl PresaleModeHandler for FixedPricePresaleHandler {
    /// Returns the remaining deposit quota for a fixed price presale.
    /// Fixed price presale cannot deposit more than the presale maximum cap.
    fn get_remaining_deposit_quota(&self, presale: &Presale, escrow: &Escrow) -> Result<u64> {
        let global_remaining_quota = presale.get_remaining_deposit_quota()?;
        let presale_registry = presale.get_presale_registry(escrow.registry_index.into())?;
        let personal_remaining_quota =
            escrow.get_remaining_deposit_quota(presale_registry.buyer_maximum_deposit_cap)?;

        Ok(global_remaining_quota.min(personal_remaining_quota))
    }

    /// Fixed price presale stop accept deposit when the presale maximum cap is reached. Therefore, can end presale immediately.
    fn end_presale_if_max_cap_reached(
        &self,
        presale: &mut Presale,
        current_timestamp: u64,
    ) -> Result<()> {
        if self.is_earlier_presale_end_once_cap_reached_disabled() {
            return Ok(());
        }

        if presale.total_deposit >= presale.presale_maximum_cap {
            presale.advance_progress_to_completed(current_timestamp)?;
        }

        Ok(())
    }

    fn process_withdraw(
        &self,
        presale: &mut Presale,
        escrow: &mut Escrow,
        amount: u64,
    ) -> Result<()> {
        presale.withdraw(escrow, amount)
    }

    fn update_pending_claim_amount(
        &self,
        presale: &Presale,
        escrow: &mut Escrow,
        current_timestamp: u64,
    ) -> Result<()> {
        let cumulative_escrow_claimable_amount =
            self.get_escrow_cumulative_claimable_token(presale, escrow, current_timestamp)?;

        let claimable_bought_token = cumulative_escrow_claimable_amount
            .safe_sub(escrow.sum_claimed_and_pending_claim_amount()?)?;

        escrow.accumulate_pending_claim_token(claimable_bought_token)?;
        escrow.update_last_refreshed_at(current_timestamp)?;

        Ok(())
    }

    fn get_total_base_token_sold(&self, presale: &Presale) -> Result<u64> {
        let mut total_sold_token: u128 = 0;

        for presale_registry in presale.presale_registries.iter() {
            if presale_registry.is_uninitialized() {
                break;
            }

            if presale_registry.total_deposit == 0 {
                continue;
            }

            total_sold_token = total_sold_token.safe_add(calculate_token_bought(
                self.q_price,
                presale_registry.total_deposit,
            )?)?;
        }

        Ok(total_sold_token.safe_cast()?)
    }

    fn get_escrow_cumulative_claimable_token(
        &self,
        presale: &Presale,
        escrow: &Escrow,
        current_timestamp: u64,
    ) -> Result<u64> {
        // 1. Calculate how many base tokens were bought
        let presale_registry = presale.get_presale_registry(escrow.registry_index.into())?;
        let total_sold_token =
            calculate_token_bought(self.q_price, presale_registry.total_deposit)?.safe_cast()?;

        // 2. Calculate how many base tokens can be claimed based on vesting schedule
        let claimable_bought_token = calculate_cumulative_claimable_amount_for_user(
            presale.immediate_release_bps,
            presale.immediate_release_timestamp,
            total_sold_token,
            presale.vesting_start_time,
            presale.vest_duration,
            current_timestamp,
            escrow.total_deposit,
            presale_registry.total_deposit,
        )?;

        Ok(claimable_bought_token)
    }

    fn suggest_deposit_amount(&self, max_deposit_amount: u64) -> Result<u64> {
        calculate_quote_token_without_surplus(self.q_price, max_deposit_amount)
    }

    fn suggest_withdraw_amount(&self, escrow: &Escrow, max_withdraw_amount: u64) -> Result<u64> {
        if escrow.total_deposit == max_withdraw_amount {
            return Ok(max_withdraw_amount);
        }
        calculate_quote_token_without_surplus(self.q_price, max_withdraw_amount)
    }

    fn can_withdraw(&self) -> bool {
        !self.is_withdraw_disabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_calculate_quote_token_without_surplus() {
        let lamport_per_price = 5;
        let q_price = lamport_per_price * SCALE_MULTIPLIER; // 5 quote token per base token

        let suggested_deposit_amount = calculate_quote_token_without_surplus(q_price, 7).unwrap();
        assert_eq!(u128::from(suggested_deposit_amount), lamport_per_price);
    }

    #[test]
    fn test_calculate_quote_token_for_base_lamport() {
        let lamport_price: u128 = 10;
        let q_price = lamport_price * SCALE_MULTIPLIER; // 10 quote token per base token

        let min_quote_amount = calculate_min_quote_amount_for_base_lamport(q_price).unwrap();
        let base_amount = calculate_token_bought(q_price, min_quote_amount).unwrap();
        assert_eq!(base_amount, 1);

        let lamport_price2: u128 = 1;
        let q_price = lamport_price2 * SCALE_MULTIPLIER; // 1 quote token per base token

        let min_quote_amount = calculate_min_quote_amount_for_base_lamport(q_price).unwrap();
        let base_amount = calculate_token_bought(q_price, min_quote_amount).unwrap();
        assert_eq!(base_amount, 1);

        let lamport_price: f64 = 0.1;
        let q_price = (lamport_price * 2.0f64.powi(SCALE_OFFSET.try_into().unwrap())) as u128;

        let min_quote_amount = calculate_min_quote_amount_for_base_lamport(q_price).unwrap();
        let base_amount = calculate_token_bought(q_price, min_quote_amount).unwrap();
        assert_eq!(base_amount, 9);
    }

    proptest! {
        #[test]
        fn test_calculate_quote_token_without_surplus_prop(lamport_per_price in 1u64..u64::MAX, max_deposit_amount in 1u64..u64::MAX) {
            let q_price = u128::from(lamport_per_price) * SCALE_MULTIPLIER; // lamport_per_price quote token per base token
            let suggested_deposit_amount = calculate_quote_token_without_surplus(q_price, max_deposit_amount).unwrap();
            assert!(suggested_deposit_amount <= max_deposit_amount);
        }
    }
}
