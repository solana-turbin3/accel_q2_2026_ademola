
// MANUAL APPROACH

use anchor_lang::prelude::*;
use crate::state::UserAccount;
use crate::utils::random::derive_random_u64;


#[derive(Accounts)]
pub struct Randomize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    /// CHECK: Read-only sysvar account containing recent slot hashes
    pub slot_hashes: UncheckedAccount<'info>,
}

impl<'info> Randomize<'info> {
    pub fn randomize(&mut self) -> Result<()> {
        let clock = Clock::get()?;

        self.user_account.data = derive_random_u64(
            &self.slot_hashes,
            clock.unix_timestamp, 
            &self.user.key());
        Ok(())
    }
}

