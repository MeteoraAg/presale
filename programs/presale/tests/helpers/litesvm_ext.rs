use anchor_client::solana_sdk::pubkey::Pubkey;
use litesvm::LiteSVM;

pub trait LiteSVMExt {
    fn get_deserialized_account<T: anchor_lang::ZeroCopy>(&self, pubkey: &Pubkey) -> Option<T>;
}

impl LiteSVMExt for LiteSVM {
    fn get_deserialized_account<T: anchor_lang::ZeroCopy>(&self, pubkey: &Pubkey) -> Option<T> {
        let account = self.get_account(pubkey)?;
        let disc = account.data.get(0..8)?;
        if T::DISCRIMINATOR != disc {
            return None;
        }

        let state_data = account.data.get(8..)?;
        bytemuck::try_pod_read_unaligned(state_data).ok()
    }
}
