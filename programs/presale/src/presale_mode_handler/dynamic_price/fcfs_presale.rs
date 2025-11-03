use crate::PresaleModeHandler;
use crate::*;

#[zero_copy]
pub struct FcfsPresaleHandler {
    pub disable_earlier_presale_end_once_cap_reached: u8,
    pub padding0: [u8; 7],
    pub padding1: [u64; 31],
}

impl FcfsPresaleHandler {
    pub fn initialize<'a>(
        presale_mode_raw_data: &'a mut [u64; 32],
        disable_earlier_presale_end_once_cap_reached: bool,
    ) -> Result<()> {
        let data = bytemuck::try_cast_slice_mut::<u64, u8>(presale_mode_raw_data)
            .map_err(|_| PresaleError::UndeterminedError)?;

        let fcfs_presale_handler = bytemuck::try_from_bytes_mut::<Self>(data)
            .map_err(|_| PresaleError::UndeterminedError)?;

        fcfs_presale_handler.disable_earlier_presale_end_once_cap_reached =
            disable_earlier_presale_end_once_cap_reached.into();

        Ok(())
    }

    pub fn is_earlier_presale_end_once_cap_reached_disabled(&self) -> bool {
        self.disable_earlier_presale_end_once_cap_reached == 1
    }
}

impl PresaleModeHandler for FcfsPresaleHandler {
    /// FCFS presale cannot deposit more than the presale maximum cap.
    fn get_remaining_deposit_quota(&self, presale: &Presale, escrow: &Escrow) -> Result<u64> {
        let global_remaining_quota = presale.get_remaining_deposit_quota()?;
        let presale_registry = presale.get_presale_registry(escrow.registry_index.into())?;
        let personal_remaining_quota =
            escrow.get_remaining_deposit_quota(presale_registry.buyer_maximum_deposit_cap)?;

        Ok(global_remaining_quota.min(personal_remaining_quota))
    }

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
        _presale: &mut Presale,
        _escrow: &mut Escrow,
        _amount: u64,
    ) -> Result<()> {
        unreachable!("FCFS presale does not support withdraw");
    }

    fn update_pending_claim_amount(
        &self,
        presale: &Presale,
        escrow: &mut Escrow,
        current_timestamp: u64,
    ) -> Result<()> {
        super::process_claim_full_presale_supply_by_share(presale, escrow, current_timestamp)
    }

    fn get_escrow_cumulative_claimable_token(
        &self,
        presale: &Presale,
        escrow: &Escrow,
        current_timestamp: u64,
    ) -> Result<u64> {
        let presale_registry = presale.get_presale_registry(escrow.registry_index.into())?;
        calculate_cumulative_claimable_amount_for_user(
            presale.immediate_release_bps,
            presale.immediate_release_timestamp,
            presale_registry.presale_supply,
            presale.vesting_start_time,
            presale.vest_duration,
            current_timestamp,
            escrow.total_deposit,
            presale_registry.total_deposit,
        )
    }

    fn get_total_base_token_sold(&self, presale: &Presale) -> Result<u64> {
        super::get_dynamic_price_based_total_base_token_sold(presale)
    }

    fn suggest_deposit_amount(&self, max_deposit_amount: u64) -> Result<u64> {
        Ok(max_deposit_amount)
    }

    fn suggest_withdraw_amount(&self, _escrow: &Escrow, _max_withdraw_amount: u64) -> Result<u64> {
        Ok(0)
    }

    fn can_withdraw(&self) -> bool {
        false
    }
}
