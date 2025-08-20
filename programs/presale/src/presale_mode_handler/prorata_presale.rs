use crate::PresaleModeHandler;
use crate::*;

pub struct ProrataPresaleHandler;

impl PresaleModeHandler for ProrataPresaleHandler {
    fn initialize_presale<'c: 'info, 'e, 'info>(
        &self,
        _presale_pubkey: Pubkey,
        presale: &mut Presale,
        presale_params: &PresaleArgs,
        presale_registries: &[PresaleRegistryArgs; MAX_PRESALE_REGISTRY_COUNT],
        locked_vesting_params: Option<&LockedVestingArgs>,
        mint_pubkeys: InitializePresaleVaultAccountPubkeys,
        _remaining_accounts: &'e mut &'c [AccountInfo<'info>],
    ) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp as u64;

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
            fixed_price_presale_params: None,
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

    fn can_withdraw(&self) -> bool {
        // Prorata presale allows withdraw
        true
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
        // Prorata presale sells the full supply of base token
        Ok(presale.presale_supply)
    }
}
