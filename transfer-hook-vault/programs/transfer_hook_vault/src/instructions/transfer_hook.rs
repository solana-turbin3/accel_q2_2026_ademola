use crate::state::{EntryStatus, WhitelistEntry};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use crate::error::VaultError;

#[derive(Accounts)]
pub struct TransferHook<'info> {
    // Accounts 0-3: fixed by the SPL transfer-hook execute interface
    // Do NOT constrain authority here — index-3 is the transfer authority
    // (owner or approved delegate), which differs from the token account owner
    // when a delegate executes the transfer (e.g. during vault withdrawals).
    #[account(token::mint = mint)]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(token::mint = mint)]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: source token account owner / transfer authority
    pub owner: UncheckedAccount<'info>,
    // Account 4: ExtraAccountMetaList (always appended by invoke_execute)
    /// CHECK: ExtraAccountMetaList
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    // Extra account 0 from meta list: this program (hook_program)
    // Included so LiteSVM can locate the program when Token-2022 does the hook CPI
    /// CHECK: this program's own account
    #[account(address = crate::ID)]
    pub hook_program: UncheckedAccount<'info>,
    // Extra account 1 from meta list: whitelist_entry PDA for the source owner
    #[account(
        seeds = [b"whitelist_entry", owner.key().as_ref()],
        bump = whitelist_entry.bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
}

impl<'info> TransferHook<'info> {
    pub fn transfer_hook(&self, _amount: u64) -> Result<()> {
        require!(
            matches!(self.whitelist_entry.status, EntryStatus::Active),
            VaultError::UserSuspended
        );
        Ok(())
    }
}
