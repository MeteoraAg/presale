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
}

pub trait PresaleModeHandler {
    fn initialize_presale<'c: 'info, 'info>(
        &self,
        presale: &mut Presale,
        tokenomic_params: &TokenomicArgs,
        presale_params: &PresaleArgs,
        locked_vesting_params: Option<&LockedVestingArgs>,
        mint_pubkeys: InitializePresaleVaultAccountPubkeys,
        remaining_accounts: &'c [AccountInfo<'info>],
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
    ) -> Result<u64>;
}

pub fn get_presale_mode_handler(presale_mode: PresaleMode) -> Box<dyn PresaleModeHandler> {
    match presale_mode {
        PresaleMode::FixedPrice => Box::new(FixedPricePresaleHandler),
        PresaleMode::Prorata => Box::new(ProrataPresaleHandler),
        PresaleMode::Fcfs => Box::new(FcfsPresaleHandler),
    }
}
