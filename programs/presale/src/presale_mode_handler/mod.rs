use std::u64;

use crate::*;

mod fixed_price_presale;
use fixed_price_presale::*;

mod prorata_presale;
use prorata_presale::*;

mod fcfs_presale;
use fcfs_presale::*;

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
    fn initialize_presale<'c: 'info, 'e, 'info>(
        &self,
        presale_pubkey: Pubkey,
        presale: &mut Presale,
        presale_params: &PresaleArgs,
        presale_registries: &[PresaleRegistryArgs],
        locked_vesting_params: Option<&LockedVestingArgs>,
        mint_pubkeys: InitializePresaleVaultAccountPubkeys,
        remaining_accounts: &'e mut &'c [AccountInfo<'info>],
    ) -> Result<()>;
    fn get_remaining_deposit_quota(&self, presale: &Presale, escrow: &Escrow) -> Result<u64>;
    fn end_presale_if_max_cap_reached(
        &self,
        presale: &mut Presale,
        current_timestamp: u64,
    ) -> Result<()>;
    fn can_withdraw(&self) -> bool;
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
    fn suggest_deposit_amount(&self, presale: &Presale, max_deposit_amount: u64) -> Result<u64>;
    fn suggest_withdraw_amount(
        &self,
        presale: &Presale,
        escrow: &Escrow,
        max_withdraw_amount: u64,
    ) -> Result<u64>;
}

pub fn get_presale_mode_handler(presale_mode: PresaleMode) -> Box<dyn PresaleModeHandler> {
    match presale_mode {
        PresaleMode::FixedPrice => Box::new(FixedPricePresaleHandler),
        PresaleMode::Prorata => Box::new(ProrataPresaleHandler),
        PresaleMode::Fcfs => Box::new(FcfsPresaleHandler),
    }
}

pub fn get_dynamic_price_based_total_base_token_sold(presale: &Presale) -> Result<u64> {
    // FCFS / Prorata presale sells the full supply of base token, but if no one deposit for the particular registry, it consider nothing been sold
    let mut total_token_sold = 0;

    for registry in presale.presale_registries.iter() {
        if registry.is_uninitialized() {
            break;
        }

        if registry.total_deposit == 0 {
            continue;
        }

        total_token_sold = total_token_sold.safe_add(registry.presale_supply)?;
    }

    Ok(total_token_sold)
}

pub fn process_claim_full_presale_supply_by_share(
    presale: &Presale,
    escrow: &mut Escrow,
    current_timestamp: u64,
) -> Result<()> {
    let presale_registry = presale.get_presale_registry(escrow.registry_index.into())?;
    let cumulative_escrow_claimable_token = calculate_cumulative_claimable_amount_for_user(
        presale.immediate_release_bps,
        presale_registry.presale_supply,
        presale.vesting_start_time,
        presale.vest_duration,
        current_timestamp,
        escrow.total_deposit,
        presale_registry.total_deposit,
    )?;

    let claimable_bought_token = cumulative_escrow_claimable_token
        .safe_sub(escrow.sum_claimed_and_pending_claim_amount()?)?;

    escrow.accumulate_pending_claim_token(claimable_bought_token)?;
    escrow.update_last_refreshed_at(current_timestamp)?;

    Ok(())
}
