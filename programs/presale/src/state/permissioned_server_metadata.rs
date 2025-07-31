use crate::*;

#[account]
pub struct PermissionedServerMetadata {
    /// Presale address
    pub presale: Pubkey,
    // padding for future use
    pub padding: [u64; 16],
    /// Server URL to retrieve the merkle proof or partially signed transaction by the operator
    pub server_url: String,
}

impl PermissionedServerMetadata {
    pub fn initialize(&mut self, presale: Pubkey, server_url: String) -> Result<()> {
        self.presale = presale;
        self.server_url = server_url;

        Ok(())
    }

    pub fn space(server_url: String) -> usize {
        std::mem::size_of::<Pubkey>()
            + std::mem::size_of::<[u64; 16]>()
            + 4
            + server_url.as_bytes().len()
    }
}
