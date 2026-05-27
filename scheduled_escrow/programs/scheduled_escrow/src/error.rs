use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Escrow has not expired yet")]
    EscrowNotExpired,
    #[msg("Expiry duration overflowed")]
    ExpiryOverflow,
}
