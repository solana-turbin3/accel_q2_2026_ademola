use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("User is suspended")]
    UserSuspended,
    #[msg("Withdrawals are suspended")]
    WithdrawsSuspended,
    #[msg("Deposit cap exceeded")]
    DepositCapExceeded,
}
