use crate::*;

mod fcfs_presale;
pub use fcfs_presale::*;

mod prorata_presale;
pub use prorata_presale::*;

fn get_dynamic_price_based_total_base_token_sold(presale: &Presale) -> Result<u64> {
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

fn process_claim_full_presale_supply_by_share(
    presale: &Presale,
    escrow: &mut Escrow,
    current_timestamp: u64,
) -> Result<()> {
    let presale_registry = presale.get_presale_registry(escrow.registry_index.into())?;
    let cumulative_escrow_claimable_token = calculate_cumulative_claimable_amount_for_user(
        presale.immediate_release_bps,
        presale.immediate_release_timestamp,
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
