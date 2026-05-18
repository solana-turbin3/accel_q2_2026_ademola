use crate::state::{Config, WhitelistEntry};
use crate::error::VaultError;
use anchor_lang::prelude::*;

// No token transfer here. The caller is responsible for executing the
// Token-2022 transfer_checked (user → vault) BEFORE calling this instruction.
// That external transfer fires the hook which enforces the whitelist.
// This instruction only validates and records the accounting update.
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"vault_config", config.admin.as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [b"whitelist_entry", user.key().as_ref()],
        bump = whitelist_entry.bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        require!(!self.config.suspended, VaultError::WithdrawsSuspended);

        if self.whitelist_entry.deposit_cap > 0 {
            require!(
                self.whitelist_entry
                    .balance_amount
                    .checked_add(amount)
                    .unwrap()
                    <= self.whitelist_entry.deposit_cap,
                VaultError::DepositCapExceeded
            );
        }

        self.whitelist_entry.balance_amount = self
            .whitelist_entry
            .balance_amount
            .checked_add(amount)
            .unwrap();

        Ok(())
    }
}
