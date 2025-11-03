use std::u64;

use crate::*;

mod fixed_price;
pub use fixed_price::*;

mod dynamic_price;
pub use dynamic_price::*;

pub struct InitializePresaleVaultAccountPubkeys {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_vault: Pubkey,
    pub quote_token_vault: Pubkey,
    pub owner: Pubkey,
    pub base: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
}

pub trait PresaleModeHandler {
    fn get_remaining_deposit_quota(&self, presale: &Presale, escrow: &Escrow) -> Result<u64>;
    fn end_presale_if_max_cap_reached(
        &self,
        presale: &mut Presale,
        current_timestamp: u64,
    ) -> Result<()>;
    fn process_withdraw(
        &self,
        presale: &mut Presale,
        escrow: &mut Escrow,
        amount: u64,
    ) -> Result<()>;
    fn update_pending_claim_amount(
        &self,
        presale: &Presale,
        escrow: &mut Escrow,
        current_timestamp: u64,
    ) -> Result<()>;
    fn get_total_base_token_sold(&self, presale: &Presale) -> Result<u64>;
    fn get_escrow_cumulative_claimable_token(
        &self,
        presale: &Presale,
        escrow: &Escrow,
        current_timestamp: u64,
    ) -> Result<u64>;
    fn suggest_deposit_amount(&self, max_deposit_amount: u64) -> Result<u64>;
    fn suggest_withdraw_amount(&self, escrow: &Escrow, max_withdraw_amount: u64) -> Result<u64>;
    fn can_withdraw(&self) -> bool;
}

pub fn enforce_dynamic_price_registries_max_buyer_cap_range(
    presale_params: &PresaleArgs,
    registries: &[PresaleRegistryArgs],
) -> Result<()> {
    for registry in registries {
        if !registry.is_uninitialized() {
            require!(
                registry.buyer_minimum_deposit_cap == 1
                    && registry.buyer_maximum_deposit_cap == presale_params.presale_maximum_cap,
                PresaleError::InvalidBuyerCapRange
            );
        }
    }
    Ok(())
}

pub fn get_presale_mode_handler(presale: &Presale) -> Result<Box<dyn PresaleModeHandler>> {
    let presale_mode = PresaleMode::from(presale.presale_mode);
    msg!("Presale mode: {:?}", presale_mode);
    let raw_data = &presale.presale_mode_raw_data;
    let raw_data_slice = bytemuck::try_cast_slice::<u64, u8>(raw_data)
        .map_err(|_| PresaleError::UndeterminedError)?;

    match presale_mode {
        PresaleMode::FixedPrice => {
            let fixed_price_presale_handler =
                *bytemuck::try_from_bytes::<FixedPricePresaleHandler>(raw_data_slice)
                    .map_err(|_| PresaleError::UndeterminedError)?;

            msg!(
                "HERE {} {} {}",
                fixed_price_presale_handler.q_price,
                fixed_price_presale_handler.disable_withdraw,
                fixed_price_presale_handler.disable_earlier_presale_end_once_cap_reached
            );

            Ok(Box::new(fixed_price_presale_handler))
        }
        PresaleMode::Prorata => Ok(Box::new(ProrataPresaleHandler)),
        PresaleMode::Fcfs => {
            let fcfs_presale_handler =
                *bytemuck::try_from_bytes::<FcfsPresaleHandler>(raw_data_slice)
                    .map_err(|_| PresaleError::UndeterminedError)?;

            Ok(Box::new(fcfs_presale_handler))
        }
    }
}
