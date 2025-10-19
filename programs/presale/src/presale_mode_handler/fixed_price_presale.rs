use crate::PresaleModeHandler;
use crate::*;

fn is_withdraw_disabled(flags: u8) -> bool {
    (flags & DISABLE_WITHDRAW_MASK) != 0
}

fn calculate_token_bought(q_price: u128, amount: u64) -> Result<u128> {
    let q_amount = u128::from(amount).safe_shl(SCALE_OFFSET)?;
    let token_bought = q_amount.safe_div(q_price)?;

    Ok(token_bought)
}

fn ensure_token_buyable(q_price: u128, amount: u64) -> Result<()> {
    let max_token_bought = calculate_token_bought(q_price, amount)?;

    require!(max_token_bought > 0, PresaleError::ZeroTokenAmount);
    require!(
        max_token_bought <= u64::MAX as u128,
        PresaleError::InvalidTokenPrice
    );
    Ok(())
}

fn ensure_enough_presale_supply(
    q_price: u128,
    presale_supply: u64,
    maximum_cap: u64,
) -> Result<()> {
    let max_presale_supply_bought = calculate_token_bought(q_price, maximum_cap)?;

    require!(
        max_presale_supply_bought <= u128::from(presale_supply),
        PresaleError::InvalidTokenPrice
    );
    Ok(())
}

fn calculate_quote_token_without_surplus(presale: &Presale, amount: u64) -> Result<u64> {
    let base_token_amount = calculate_token_bought(presale.fixed_price_presale_q_price, amount)?;

    let quote_token_needed = base_token_amount
        .safe_mul(presale.fixed_price_presale_q_price)?
        .div_ceil(SCALE_MULTIPLIER);

    let quote_token_needed: u64 = quote_token_needed.safe_cast()?;

    // This should never happen
    require!(
        quote_token_needed <= amount,
        PresaleError::UndeterminedError
    );

    Ok(quote_token_needed)
}

pub struct FixedPricePresaleHandler;

impl PresaleModeHandler for FixedPricePresaleHandler {
    fn initialize_presale<'c: 'info, 'e, 'info>(
        &self,
        presale_pubkey: Pubkey,
        presale: &mut Presale,
        presale_params: &PresaleArgs,
        presale_registries: &[PresaleRegistryArgs],
        locked_vesting_params: Option<&LockedVestingArgs>,
        mint_pubkeys: InitializePresaleVaultAccountPubkeys,
        remaining_accounts: &'e mut &'c [AccountInfo<'info>],
    ) -> Result<()> {
        // 1. Get extra params about fixed price presale mode
        let slice = remaining_accounts.split_first();

        let Some((presale_extra_param_ai, remaining_account_slice)) = slice else {
            return Err(PresaleError::MissingPresaleExtraParams.into());
        };

        *remaining_accounts = remaining_account_slice;

        let presale_extra_param_al =
            AccountLoader::<FixedPricePresaleExtraArgs>::try_from(presale_extra_param_ai)?;

        let presale_extra_param = presale_extra_param_al.load()?;
        require!(
            presale_extra_param.presale == presale_pubkey,
            PresaleError::MissingPresaleExtraParams
        );

        // 2. Validate fixed price presale parameters
        // TODO: Should we make sure there's no impossible to fill gap?
        // For example: 1 token = 1 USDC, presale_maximum_cap = 100 USDC, buyer_minimum_deposit_cap = 20 USDC, buyer_maximum_deposit_cap = 90 USDC
        // User 1 deposit 90 USDC, remaining_presale_cap = 100 - 90 = 10
        // But buyer_minimum_deposit_cap = 20, thus it's impossible to fill the gap
        for registry in presale_registries {
            if !registry.is_uninitialized() {
                // ensure buyer_minimum_deposit_cap and buyer_maximum_deposit_cap can buy at least 1 token and not exceed u64::MAX token
                ensure_token_buyable(
                    presale_extra_param.q_price,
                    registry.buyer_minimum_deposit_cap,
                )?;
                ensure_token_buyable(
                    presale_extra_param.q_price,
                    registry.buyer_maximum_deposit_cap,
                )?;
            }
        }

        let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;

        let InitializePresaleVaultAccountPubkeys {
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
            base,
            base_token_program,
            quote_token_program,
        } = mint_pubkeys;

        // 3. Create presale vault
        presale.initialize(PresaleInitializeArgs {
            presale_params: *presale_params,
            presale_registries,
            locked_vesting_params: locked_vesting_params.cloned(),
            fixed_price_presale_params: Some(*presale_extra_param),
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
            current_timestamp,
            base,
            base_token_program,
            quote_token_program,
        })?;

        // Ensure presale supply is enough to fulfill presale maximum cap
        ensure_enough_presale_supply(
            presale_extra_param.q_price,
            presale.presale_supply,
            presale_params.presale_maximum_cap,
        )?;

        Ok(())
    }

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
        super::end_presale_if_max_cap_reached(presale, current_timestamp)
    }

    fn can_withdraw(&self, presale: &Presale) -> bool {
        // Fixed price presale allow withdraw
        !is_withdraw_disabled(presale.fixed_price_presale_flags)
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
                presale.fixed_price_presale_q_price,
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
        let total_sold_token = calculate_token_bought(
            presale.fixed_price_presale_q_price,
            presale_registry.total_deposit,
        )?
        .safe_cast()?;

        // 2. Calculate how many base tokens can be claimed based on vesting schedule
        let claimable_bought_token = calculate_cumulative_claimable_amount_for_user(
            presale.immediate_release_bps,
            total_sold_token,
            presale.vesting_start_time,
            presale.vest_duration,
            current_timestamp,
            escrow.total_deposit,
            presale_registry.total_deposit,
        )?;

        Ok(claimable_bought_token)
    }

    fn suggest_deposit_amount(&self, presale: &Presale, max_deposit_amount: u64) -> Result<u64> {
        calculate_quote_token_without_surplus(presale, max_deposit_amount)
    }

    fn suggest_withdraw_amount(
        &self,
        presale: &Presale,
        escrow: &Escrow,
        max_withdraw_amount: u64,
    ) -> Result<u64> {
        if escrow.total_deposit == max_withdraw_amount {
            return Ok(max_withdraw_amount);
        }
        calculate_quote_token_without_surplus(presale, max_withdraw_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_calculate_quote_token_without_surplus() {
        let mut presale = Presale::default();
        let lamport_per_price = 5;
        presale.fixed_price_presale_q_price = lamport_per_price * SCALE_MULTIPLIER; // 5 quote token per base token

        let suggested_deposit_amount = calculate_quote_token_without_surplus(&presale, 7).unwrap();
        assert_eq!(u128::from(suggested_deposit_amount), lamport_per_price);
    }

    proptest! {
        #[test]
        fn test_calculate_quote_token_without_surplus_prop(lamport_per_price in 1u64..u64::MAX, max_deposit_amount in 1u64..u64::MAX) {
            let mut presale = Presale::default();
            presale.fixed_price_presale_q_price = u128::from(lamport_per_price) * SCALE_MULTIPLIER; // lamport_per_price quote token per base token
            let suggested_deposit_amount = calculate_quote_token_without_surplus(&presale, max_deposit_amount).unwrap();
            assert!(suggested_deposit_amount <= max_deposit_amount);
        }
    }
}
