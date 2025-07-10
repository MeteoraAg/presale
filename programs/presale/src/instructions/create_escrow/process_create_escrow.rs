use crate::*;

pub struct HandleCreateEscrowArgs<'a, 'b, 'c> {
    pub presale: &'a Presale,
    pub escrow: &'b AccountLoader<'c, Escrow>,
    pub presale_pubkey: Pubkey,
    pub owner_pubkey: Pubkey,
}

pub fn process_create_escrow(args: HandleCreateEscrowArgs) -> Result<()> {
    let HandleCreateEscrowArgs {
        presale,
        escrow,
        presale_pubkey,
        owner_pubkey,
    } = args;

    // 1. Ensure presale is open for deposit
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let progress = presale.get_presale_progress(current_timestamp);
    require!(
        progress == PresaleProgress::Ongoing,
        PresaleError::PresaleNotOpenForDeposit
    );

    // 2. Initialize the escrow account
    let mut escrow = escrow.load_init()?;
    escrow.initialize(presale_pubkey, owner_pubkey, current_timestamp)?;

    Ok(())
}
