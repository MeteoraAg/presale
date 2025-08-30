use crate::*;

#[error_code]
#[derive(PartialEq)]
pub enum PresaleError {
    #[msg("Invalid mint metadata")]
    InvalidMintMetadata,

    #[msg("Invalid token info")]
    InvalidTokenInfo,

    #[msg("Invalid token supply")]
    InvalidTokenSupply,

    #[msg("Invalid presale info")]
    InvalidPresaleInfo,

    #[msg("Invalid quote mint")]
    InvalidQuoteMint,

    #[msg("Invalid base mint")]
    InvalidBaseMint,

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

    #[msg("Presale is not open for withdraw")]
    PresaleNotOpenForWithdraw,

    #[msg("Presale is not open for withdraw remaining quote")]
    PresaleNotOpenForWithdrawRemainingQuote,

    #[msg("Invalid presale whitelist mode")]
    InvalidPresaleWhitelistMode,

    #[msg("Presale is ended")]
    PresaleEnded,

    #[msg("Presale is not open for claim")]
    PresaleNotOpenForClaim,

    #[msg("Invalid merkle proof")]
    InvalidMerkleProof,

    #[msg("Deposit amount out of cap")]
    DepositAmountOutOfCap,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Insufficient escrow balance")]
    InsufficientEscrowBalance,

    #[msg("Remaining quote has already been withdrawn")]
    RemainingQuoteAlreadyWithdrawn,

    #[msg("Presale not completed")]
    PresaleNotCompleted,

    #[msg("No unsold tokens")]
    NoUnsoldTokens,

    #[msg("Escrow is not empty")]
    EscrowNotEmpty,

    #[msg("Invalid unsold token action")]
    InvalidUnsoldTokenAction,

    #[msg("Creator has already withdrawn")]
    CreatorAlreadyWithdrawn,

    #[msg("Escrow not refreshed")]
    EscrowNotRefreshed,

    #[msg("Undetermined error")]
    UndeterminedError,

    #[msg("Invalid token vault")]
    InvalidTokenVault,

    #[msg("Invalid remaining account slice")]
    InvalidRemainingAccountSlice,

    #[msg("Duplicated remaining account types")]
    DuplicatedRemainingAccountTypes,

    #[msg("Missing remaining account for transfer hook")]
    MissingRemainingAccountForTransferHook,

    #[msg("No transfer hook program")]
    NoTransferHookProgram,

    #[msg("Invalid operator")]
    InvalidOperator,

    #[msg("No unsold base tokens")]
    NoUnsoldBaseTokens,

    #[msg("Unsold base token action already performed")]
    UnsoldBaseTokenActionAlreadyPerformed,

    #[msg("Invalid presale registry index")]
    InvalidPresaleRegistryIndex,

    #[msg("Multiple presale registries are not allowed")]
    MultiplePresaleRegistriesNotAllowed,

    #[msg("Invalid deposit cap")]
    InvalidDepositCap,

    #[msg("Presale is not open for collect fee")]
    PresaleNotOpenForCollectFee,
}
