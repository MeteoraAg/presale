use crate::*;
use static_assertions::const_assert_eq;

#[account(zero_copy)]
#[derive(InitSpace, Debug)]
pub struct MerkleRootConfig {
    /// The 256-bit merkle root.
    pub root: [u8; 32],
    /// Padding for future use
    pub padding0: u64,
    /// Presale pubkey that config is belong
    pub presale: Pubkey,
    /// Version
    pub version: u64,
    /// Padding for further use
    pub padding1: [u128; 4],
}

static_assertions::const_assert_eq!(MerkleRootConfig::INIT_SPACE, 144);
static_assertions::assert_eq_align!(MerkleRootConfig, u128);

impl MerkleRootConfig {
    pub fn initialize(&mut self, presale: Pubkey, root: [u8; 32], version: u64) {
        self.presale = presale;
        self.root = root;
        self.version = version;
    }
}

const_assert_eq!(std::mem::size_of::<MerkleRootConfig>(), 144);
