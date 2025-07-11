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
        if presale.total_deposit == presale.presale_maximum_cap {
            presale.update_presale_end_time(current_timestamp);
        }

        Ok(())
    }
}
