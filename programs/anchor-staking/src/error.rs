use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Freeze period not passed")]
    FreezePeriodNotPassed,
    #[msg("Invalid Admin")]
    InvalidAdmin,
    #[msg("overflow")]
    Overflow,
    #[msg("underflow")]
    Underflow,
    #[msg("Stake Period too low")]
    TooLowStakePeriod,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("max NFTs staked already")]
    ExceedsMaxNftStake,
    #[msg("metadata mint does not match the nft mint")]
    MetadataMintMismatch,
    #[msg("there is no specified collection")]
    MissingCollection,
    #[msg("the nft does not belong to specified collection")]
    WrongCollection,
    #[msg("collection is not verified")]
    CollectionNotVerified,
    #[msg("invalid mint supply")]
    InvalidMintDecimals,
    #[msg("invalid NFT supple")]
    InvalidNftSupply,
    #[msg("Invalid token owner")]
    InvalidTokenOwner,
    #[msg("Invalid Token amount")]
    InvalidTokenAmount,
}
