use serde::{Deserialize, Serialize};
use solana_program::{hash::hashv, pubkey::Pubkey};
use solana_sdk::hash::Hash;

/// Represents the escrow information for an account.
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    /// Pubkey of the escrow owner
    pub escrow_owner: Pubkey,
    /// Presale registry index
    pub registry_index: u8,
    /// Personal deposit cap
    pub deposit_cap: u64,
    /// Escrow owner proof of inclusion in the Merkle Tree
    pub proof: Option<Vec<[u8; 32]>>,
}

impl TreeNode {
    pub fn hash(&self) -> Hash {
        hashv(&[
            &self.escrow_owner.to_bytes(),
            &self.registry_index.to_le_bytes(),
            &self.deposit_cap.to_le_bytes(),
        ])
    }
}
