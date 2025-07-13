use core::panic;

use crate::PresaleModeHandler;
use crate::*;

fn ensure_token_buyable(q_price: u128, amount: u64) -> Result<()> {
    let q_amount = u128::from(amount).checked_shl(64).unwrap();
    let max_token_bought = q_amount.checked_div(q_price).unwrap();

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
    let q_amount = u128::from(maximum_cap).checked_shl(64).unwrap();
    let max_presale_supply_bought = q_amount.checked_div(q_price).unwrap();

    msg!("max_presale_supply_bought: {}", max_presale_supply_bought);
    msg!("presale_supply: {}", presale_supply);

    require!(
        max_presale_supply_bought <= u128::from(presale_supply),
        PresaleError::InvalidTokenPrice
    );
    Ok(())
}

pub struct FixedPricePresaleHandler;

impl PresaleModeHandler for FixedPricePresaleHandler {
    fn initialize_presale<'c: 'info, 'info>(
        &self,
        presale: &mut Presale,
        tokenomic_params: &TokenomicArgs,
        presale_params: &PresaleArgs,
        locked_vesting_params: Option<&LockedVestingArgs>,
        mint_pubkeys: InitializePresaleVaultAccountPubkeys,
        remaining_accounts: &'c [AccountInfo<'info>],
    ) -> Result<()> {
        // 1. Get extra params about fixed price presale mode
        let presale_extra_param_ai = remaining_accounts
            .first()
            .ok_or_else(|| error!(PresaleError::MissingPresaleExtraParams))?;

        let presale_extra_param_al =
            AccountLoader::<FixedPricePresaleExtraArgs>::try_from(presale_extra_param_ai)?;

        let presale_extra_param = presale_extra_param_al.load()?;

        // 2. Validate fixed price presale parameters
        ensure_token_buyable(
            presale_extra_param.q_price,
            presale_params.buyer_maximum_deposit_cap,
        )?;

        ensure_enough_presale_supply(
            presale_extra_param.q_price,
            tokenomic_params.presale_pool_supply,
            presale_params.presale_maximum_cap,
        )?;

        let current_timestamp = Clock::get()?.unix_timestamp as u64;

        let InitializePresaleVaultAccountPubkeys {
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
        } = mint_pubkeys;

        // 3. Create presale vault
        presale.initialize(PresaleInitializeArgs {
            tokenomic_params: *tokenomic_params,
            presale_params: *presale_params,
            locked_vesting_params: locked_vesting_params.cloned(),
            fixed_price_presale_params: Some(*presale_extra_param),
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
            current_timestamp,
        });

        Ok(())
    }

    /// Returns the remaining deposit quota for a fixed price presale.
    /// Fixed price presale cannot deposit more than the presale maximum cap.
    fn get_remaining_deposit_quota(&self, presale: &Presale, escrow: &Escrow) -> Result<u64> {
        let global_remaining_quota = presale.get_remaining_deposit_quota()?;
        let personal_remaining_quota =
            escrow.get_remaining_deposit_quota(presale.buyer_maximum_deposit_cap)?;

        Ok(global_remaining_quota.min(personal_remaining_quota))
    }

    /// Fixed price presale stop accept deposit when the presale maximum cap is reached. Therefore, can end presale immediately.
    fn end_presale_if_max_cap_reached(
        &self,
        presale: &mut Presale,
        current_timestamp: u64,
    ) -> Result<()> {
        if presale.total_deposit >= presale.presale_maximum_cap {
            presale.advance_progress_to_completed(current_timestamp)?;
        }

        Ok(())
    }

    fn can_withdraw(&self) -> bool {
        // Fixed price presale allow withdraw
        true
    }

    fn process_withdraw(
        &self,
        presale: &mut Presale,
        escrow: &mut Escrow,
        amount: u64,
    ) -> Result<u64> {
        presale.withdraw(escrow, amount)
    }

    fn process_claim(
        &self,
        presale: &mut Presale,
        escrow: &mut Escrow,
        current_timestamp: u64,
    ) -> Result<u64> {
        // 1. Calculate how many base tokens were bought
        let q_total_deposit = u128::from(presale.total_deposit).checked_shl(64).unwrap();
        let total_sold_token = q_total_deposit
            .checked_div(presale.fixed_price_presale_q_price)
            .unwrap();

        // 2. Calculate how many base tokens can be claimed based on vesting schedule
        let vesting_start_time = presale.lock_end_time;
        let elapsed_seconds = current_timestamp
            .checked_sub(vesting_start_time)
            .unwrap()
            .min(presale.vest_duration);

        let dripped_total_sold_token = total_sold_token
            .checked_mul(elapsed_seconds.into())
            .unwrap()
            .checked_div(presale.vest_duration.into())
            .unwrap();

        // 3. Calculate how many base tokens can be claimed by the escrow
        let dripped_escrow_bought_token = dripped_total_sold_token
            .checked_mul(escrow.total_deposit.into())
            .unwrap()
            .checked_div(presale.total_deposit.into())
            .unwrap();

        let claimable_bought_token: u64 = dripped_escrow_bought_token
            .checked_sub(escrow.total_claimed_token.into())
            .unwrap()
            .try_into()
            .unwrap();

        if claimable_bought_token > 0 {
            // 4. Update presale and escrow state
            presale.claim(escrow, claimable_bought_token)?;
        }

        Ok(claimable_bought_token)
    }
}
