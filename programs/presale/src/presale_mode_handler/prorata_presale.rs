use crate::PresaleModeHandler;
use crate::*;

pub struct ProrataPresaleHandler;

impl PresaleModeHandler for ProrataPresaleHandler {
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
            fixed_price_presale_params: None,
            base_mint,
            quote_mint,
            base_token_vault,
            quote_token_vault,
            owner,
            current_timestamp,
        });

        Ok(())
    }

    fn get_remaining_deposit_quota(&self, presale: &Presale, escrow: &Escrow) -> Result<u64> {
        escrow.get_remaining_deposit_quota(presale.buyer_maximum_deposit_cap)
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
    ) -> Result<u64> {
        presale.withdraw(escrow, amount)
    }

    fn process_claim(
        &self,
        presale: &mut Presale,
        escrow: &mut Escrow,
        current_timestamp: u64,
    ) -> Result<u64> {
        process_claim_full_presale_supply_by_share(presale, escrow, current_timestamp)
    }

    fn get_escrow_dripped_bought_token(
        &self,
        presale: &Presale,
        escrow: &Escrow,
        current_timestamp: u64,
    ) -> Result<u128> {
        get_dripped_escrow_bought_token_by_share(presale, escrow, current_timestamp)
    }

    fn get_total_base_token_sold(&self, presale: &Presale) -> Result<u64> {
        // Prorata presale sells the full supply of base token
        Ok(presale.presale_supply)
    }
}
