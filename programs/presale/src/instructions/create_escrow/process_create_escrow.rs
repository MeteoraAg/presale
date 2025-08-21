use crate::*;

pub struct HandleCreateEscrowArgs<'a, 'b, 'c> {
    pub presale: &'a mut Presale,
    pub escrow: &'b AccountLoader<'c, Escrow>,
    pub presale_pubkey: Pubkey,
    pub owner_pubkey: Pubkey,
    pub registry_index: u8,
}

pub fn process_create_escrow(args: HandleCreateEscrowArgs) -> Result<()> {
    let HandleCreateEscrowArgs {
        presale,
        escrow,
        presale_pubkey,
        owner_pubkey,
        registry_index,
    } = args;

    // 1. Ensure presale is open for deposit
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let progress = presale.get_presale_progress(current_timestamp);
    require!(
        progress == PresaleProgress::Ongoing,
        PresaleError::PresaleNotOpenForDeposit
    );

    // 2. Ensure valid registry index
    require!(
        registry_index < presale.total_presale_registry_count,
        PresaleError::InvalidPresaleRegistryIndex
    );

    // 3. Initialize the escrow account
    let mut escrow = escrow.load_init()?;
    escrow.initialize(
        presale_pubkey,
        owner_pubkey,
        current_timestamp,
        registry_index,
    )?;

    // 4. Update the presale state
    presale.increase_escrow_count(registry_index)?;

    Ok(())
}
