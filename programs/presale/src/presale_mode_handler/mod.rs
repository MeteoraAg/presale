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
        presale_registries: &[PresaleRegistryArgs; MAX_PRESALE_REGISTRY_COUNT],
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
    fn get_escrow_dripped_bought_token(
        &self,
        presale: &Presale,
        escrow: &Escrow,
        current_timestamp: u64,
    ) -> Result<u128>;
}

pub fn get_presale_mode_handler(presale_mode: PresaleMode) -> Box<dyn PresaleModeHandler> {
    match presale_mode {
        PresaleMode::FixedPrice => Box::new(FixedPricePresaleHandler),
        PresaleMode::Prorata => Box::new(ProrataPresaleHandler),
        PresaleMode::Fcfs => Box::new(FcfsPresaleHandler),
    }
}

pub fn process_claim_full_presale_supply_by_share(
    presale: &Presale,
    escrow: &mut Escrow,
    current_timestamp: u64,
) -> Result<()> {
    let dripped_escrow_bought_token = calculate_dripped_amount_for_user(
        presale.lock_end_time,
        presale.vest_duration,
        current_timestamp,
        presale.presale_supply,
        escrow.total_deposit,
        presale.total_deposit,
    )?;

    let claimable_bought_token: u64 = dripped_escrow_bought_token
        .safe_sub(escrow.sum_claimed_and_pending_claim_amount()?.into())?
        .safe_cast()?;

    escrow.accumulate_pending_claim_token(claimable_bought_token)?;
    escrow.update_last_refreshed_at(current_timestamp)?;

    Ok(())
}
