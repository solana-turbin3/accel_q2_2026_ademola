use crate::state::{Config, EntryStatus, WhitelistEntry};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct WhitelistUser<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault_config", admin.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: The user account/address to be whitelisted.
    #[account(mut)]
    pub user: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"whitelist_entry", user.key().as_ref()],
        space = 8 + WhitelistEntry::INIT_SPACE,
        bump
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    pub system_program: Program<'info, System>,
}

impl<'info> WhitelistUser<'info> {
    pub fn whitelist(&mut self, deposit_cap: u64, bumps: &WhitelistUserBumps) -> Result<()> {
        self.whitelist_entry.set_inner(WhitelistEntry {
            balance_amount: 0,
            deposit_cap: deposit_cap,
            bump: bumps.whitelist_entry,
            status: EntryStatus::Active,
        });
        Ok(())
    }
}
