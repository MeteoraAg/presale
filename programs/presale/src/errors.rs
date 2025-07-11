use crate::*;

#[error_code]
#[derive(PartialEq)]
pub enum PresaleError {
    #[msg("Invalid token info")]
    InvalidTokenInfo,

    #[msg("Invalid token supply")]
    InvalidTokenSupply,

    #[msg("Invalid presale info")]
    InvalidPresaleInfo,

    #[msg("Invalid quote mint")]
    InvalidQuoteMint,

    #[msg("Invalid lock vesting info")]
    InvalidLockVestingInfo,

    #[msg("Invalid token price")]
    InvalidTokenPrice,

    #[msg("Missing presale extra params account")]
    MissingPresaleExtraParams,

    #[msg("Zero token amount")]
    ZeroTokenAmount,

    #[msg("Token2022 extensions or native mint is not supported")]
    UnsupportedToken2022MintOrExtension,

    #[msg("Invalid creator account")]
    InvalidCreatorAccount,

    #[msg("Presale is not open for deposit")]
    PresaleNotOpenForDeposit,

    #[msg("Invalid presale whitelist mode")]
    InvalidPresaleWhitelistMode,

    #[msg("Presale is ended")]
    PresaleEnded,

    #[msg("Invalid merkle proof")]
    InvalidMerkleProof,

    #[msg("Deposit amount out of cap")]
    DepositAmountOutOfCap,

    #[msg("Math overflow")]
    MathOverflow,
}
