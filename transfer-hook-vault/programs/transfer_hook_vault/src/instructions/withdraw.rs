use crate::state::{Config, EntryStatus, WhitelistEntry};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{approve_checked, ApproveChecked, Mint, TokenAccount, TokenInterface},
};
use crate::error::VaultError;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"vault_config", config.admin.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"whitelist_entry", user.key().as_ref()],
        bump = whitelist_entry.bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(!self.config.suspended, VaultError::WithdrawsSuspended);
        require!(
            matches!(self.whitelist_entry.status, EntryStatus::Active),
            VaultError::UserSuspended
        );
        require!(
            self.whitelist_entry.balance_amount >= amount,
            VaultError::InsufficientBalance
        );

        let admin_key = self.config.admin;
        let bump = self.config.bump;
        let signer_seeds: &[&[&[u8]]] = &[&[b"vault_config", admin_key.as_ref(), &[bump]]];

        // Approve the user as delegate for exactly `amount` tokens from the vault.
        // approve_checked does NOT fire the transfer hook. The user then executes
        // transfer_checked (vault → user) outside this instruction, which does
        // fire the hook and re-validates the whitelist without re-entrancy.
        approve_checked(
            CpiContext::new_with_signer(
                self.token_program.key(),
                ApproveChecked {
                    to: self.vault.to_account_info(),
                    mint: self.mint.to_account_info(),
                    delegate: self.user.to_account_info(),
                    authority: self.config.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
            self.mint.decimals,
        )?;

        self.whitelist_entry.balance_amount = self
            .whitelist_entry
            .balance_amount
            .checked_sub(amount)
            .unwrap();

        Ok(())
    }
}
