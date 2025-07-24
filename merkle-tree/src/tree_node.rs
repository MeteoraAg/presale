use serde::{Deserialize, Serialize};
use solana_program::{hash::hashv, pubkey::Pubkey};
use solana_sdk::hash::Hash;

/// Represents the escrow information for an account.
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TreeNode {
    /// Pubkey of the escrow owner
    pub escrow_owner: Pubkey,
    /// Escrow owner proof of inclusion in the Merkle Tree
    pub proof: Option<Vec<[u8; 32]>>,
}

impl TreeNode {
    pub fn hash(&self) -> Hash {
        hashv(&[&self.escrow_owner.to_bytes()])
    }
}
