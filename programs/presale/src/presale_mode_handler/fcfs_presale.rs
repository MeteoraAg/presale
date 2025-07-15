use crate::PresaleModeHandler;
use crate::*;

pub struct FcfsPresaleHandler;

impl PresaleModeHandler for FcfsPresaleHandler {
    fn initialize_presale<'c: 'info, 'info>(
        &self,
        presale: &mut Presale,
        tokenomic_params: &TokenomicArgs,
        presale_params: &PresaleArgs,
        locked_vesting_params: Option<&LockedVestingArgs>,
        mint_pubkeys: InitializePresaleVaultAccountPubkeys,
        _remaining_accounts: &'c [AccountInfo<'info>],
    ) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp as u64;

        if let Some(lock) = locked_vesting_params {
            // Ensure cannot lock unsold token since prorata presale will sold all supply
            require!(
                lock.lock_unsold_token == 0,
                PresaleError::InvalidUnsoldTokenAction
            );
        }

        let InitializePresaleVaultAccountPubkeys {
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
        } = mint_pubkeys;

        presale.initialize(PresaleInitializeArgs {
            tokenomic_params: *tokenomic_params,
            presale_params: *presale_params,
            locked_vesting_params: locked_vesting_params.cloned(),
            fixed_price_presale_params: None,
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
            current_timestamp,
        })?;

        Ok(())
    }

    fn get_remaining_deposit_quota(&self, presale: &Presale, escrow: &Escrow) -> Result<u64> {
        let global_remaining_quota = presale.get_remaining_deposit_quota()?;
        let personal_remaining_quota =
            escrow.get_remaining_deposit_quota(presale.buyer_maximum_deposit_cap)?;

        Ok(global_remaining_quota.min(personal_remaining_quota))
    }

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
        // FCFS do not allow withdraw
        false
    }

    fn process_withdraw(
        &self,
        _presale: &mut Presale,
        _escrow: &mut Escrow,
        _amount: u64,
    ) -> Result<u64> {
        unreachable!("FCFS presale does not support withdraw");
    }

    fn update_pending_claim_amount(
        &self,
        presale: &Presale,
        escrow: &mut Escrow,
        current_timestamp: u64,
    ) -> Result<()> {
        process_claim_full_presale_supply_by_share(presale, escrow, current_timestamp)
    }

    fn get_escrow_dripped_bought_token(
        &self,
        presale: &Presale,
        escrow: &Escrow,
        current_timestamp: u64,
    ) -> Result<u128> {
        calculate_dripped_amount_for_user(
            presale.lock_end_time,
            presale.vest_duration,
            current_timestamp,
            presale.presale_supply,
            escrow.total_deposit,
            presale.total_deposit,
        )
    }

    fn get_total_base_token_sold(&self, presale: &Presale) -> Result<u64> {
        // FCFS presale sells the full supply of base token
        Ok(presale.presale_supply)
    }
}
