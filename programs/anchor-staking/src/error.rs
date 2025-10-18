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
}
