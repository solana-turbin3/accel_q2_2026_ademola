use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::{anchor::commit, ephem::commit_accounts};

use crate::state::UserAccount;
use crate::utils::random::derive_random_u64;

#[commit]
#[derive(Accounts)]
pub struct RandomizeCommit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,
    /// CHECK: Read-only sysvar account containing recent slot hashes
    pub slot_hashes: UncheckedAccount<'info>,
}


impl<'info>RandomizeCommit<'info> {
    pub fn randomize_commit(&mut self) -> Result<()> {
        let clock = Clock::get()?;

        self.user_account.data = derive_random_u64(
            &self.slot_hashes,
            clock.unix_timestamp,
            &self.user.key()
        );
        
        commit_accounts(
            &self.user.to_account_info(),
            vec![&self.user_account.to_account_info()],
            &self.magic_context,
            &self.magic_program
        )?;


        Ok(())
    }
}