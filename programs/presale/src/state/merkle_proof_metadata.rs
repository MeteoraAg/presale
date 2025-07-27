use crate::*;

#[account]
pub struct MerkleProofMetadata {
    /// Presale address
    pub presale: Pubkey,
    // padding for future use
    pub padding: [u64; 16],
    /// Merkle root proof URL
    pub proof_url: String,
}

impl MerkleProofMetadata {
    pub fn initialize(&mut self, presale: Pubkey, proof_url: String) -> Result<()> {
        self.presale = presale;
        self.proof_url = proof_url;

        Ok(())
    }

    pub fn space(proof_url: String) -> usize {
        std::mem::size_of::<Pubkey>()
            + std::mem::size_of::<[u64; 16]>()
            + 4
            + proof_url.as_bytes().len()
    }
}
