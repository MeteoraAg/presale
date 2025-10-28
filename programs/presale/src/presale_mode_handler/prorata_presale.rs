use crate::PresaleModeHandler;
use crate::*;

pub struct ProrataPresaleHandler;

impl PresaleModeHandler for ProrataPresaleHandler {
    fn initialize_presale<'c: 'info, 'e, 'info>(
        &self,
        presale: &mut Presale,
        common_args: &'e CommonPresaleArgs,
        mint_pubkeys: InitializePresaleVaultAccountPubkeys,
        disable_withdraw: bool,
        q_price: u128,
        disable_earlier_presale_end_once_cap_reached: bool,
    ) -> Result<()> {
        let CommonPresaleArgs {
            presale_params,
            presale_registries,
            ..
        } = common_args;

        let current_timestamp: u64 = Clock::get()?.unix_timestamp.safe_cast()?;

        let whitelist_mode = WhitelistMode::from(presale_params.whitelist_mode);
        if whitelist_mode.is_permissioned() {
            enforce_dynamic_price_registries_max_buyer_cap_range(
                presale_params,
                presale_registries,
            )?;
        }

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
            common_args,
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
            current_timestamp,
            base,
            base_token_program,
            quote_token_program,
            disable_earlier_presale_end_once_cap_reached,
            disable_withdraw,
            q_price,
            presale_mode: PresaleMode::Prorata,
        })?;

        Ok(())
    }

    fn get_remaining_deposit_quota(&self, presale: &Presale, escrow: &Escrow) -> Result<u64> {
        // Prorata can deposit > presale maximum cap. Therefore, the remaining deposit quota is the quote leftover in the escrow.
        let presale_registry = presale.get_presale_registry(escrow.registry_index.into())?;
        escrow.get_remaining_deposit_quota(presale_registry.buyer_maximum_deposit_cap)
    }

    fn end_presale_if_max_cap_reached(
        &self,
        _presale: &mut Presale,
        _current_timestamp: u64,
    ) -> Result<()> {
        // Do nothing because prorata allow over subscription
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
        process_claim_full_presale_supply_by_share(presale, escrow, current_timestamp)
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
        get_dynamic_price_based_total_base_token_sold(presale)
    }

    fn suggest_deposit_amount(&self, _presale: &Presale, max_deposit_amount: u64) -> Result<u64> {
        Ok(max_deposit_amount)
    }

    fn suggest_withdraw_amount(
        &self,
        _presale: &Presale,
        _escrow: &Escrow,
        max_withdraw_amount: u64,
    ) -> Result<u64> {
        Ok(max_withdraw_amount)
    }
}
