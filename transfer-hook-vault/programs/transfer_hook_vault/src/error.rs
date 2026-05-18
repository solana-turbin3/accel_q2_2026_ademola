use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("User is suspended")]
    UserSuspended,
    #[msg("User is not whitelisted")]
    UserNotWhitelisted,
    #[msg("Withdrawals are suspended")]
    WithdrawsSuspended,
    #[msg("Deposit cap exceeded")]
    DepositCapExceeded,
    #[msg("Insufficient vault balance")]
    InsufficientBalance,
    #[msg("User still has funds in the vault — withdraw first")]
    UserHasBalance,
}
