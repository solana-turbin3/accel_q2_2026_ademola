use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    /// CHECK: user account to remove from whitelist
    pub user: UncheckedAccount<'info>,

    #[account(
        seeds = [b"vault_config", admin.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        close = admin,
        seeds = [b"whitelist_entry", user.key().as_ref()],
        bump = whitelist_entry.bump
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    pub system_program: Program<'info, System>,
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove(&mut self) -> Result<()> {
        Ok(())
    }
}
