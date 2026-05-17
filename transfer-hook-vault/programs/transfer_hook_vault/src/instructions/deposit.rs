use crate::state::{Config, EntryStatus, WhitelistEntry};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        seeds = [b"vault_config", config.admin.as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
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
    /// CHECK: ExtraAccountMetaList
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {

        require!(
            self.config.suspended == false,
            VaultError::WithdrawsSuspended
        );

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

        transfer_checked(
            CpiContext::new(
                self.token_program.key(),
                TransferChecked {
                    from: self.user_token_account.to_account_info(),
                    mint: self.mint.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.user.to_account_info(),
                },
            )
            .with_remaining_accounts(vec![
                self.extra_account_meta_list.to_account_info(),
                self.whitelist_entry.to_account_info(),
            ]),
            amount,
            self.mint.decimals,
        )?;

        self.whitelist_entry.balance_amount = self
            .whitelist_entry
            .balance_amount
            .checked_add(amount)
            .unwrap();

        Ok(())
    }
}
