use crate::state::{Config, EntryStatus, WhitelistEntry};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, TokenAccount, TokenInterface, TransferChecked,
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"vault_config", config.admin.key().as_ref()],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_associated_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    #[account(
        seeds = [b"whitelist_entry", user.key().as_ref()],
        bump = whitelist_entry.bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: ExtraAccountMetaList Account,
    #[account(
            seeds = [b"extra-account-metas", mint.key().as_ref()],
            bump
        )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&self, amount: u64) -> Result<()> {
        require!(
            self.config.suspended == false,
            VaultError::WithdrawsSuspended
        );
        require!(
            matches!(self.whitelist_entry.status, EntryStatus::Active),
            VaultError::UserSuspended
        );

        let signer_seeds: &[&[&[u8]]] = &[&[
            b"vault_config",
            self.config.admin.as_ref(),
            &[self.config.bump],
        ]];

        transfer_checked(
            CpiContext::new_with_signer(
                self.token_program.key(),
                TransferChecked {
                    from: self.vault.to_account_info(),
                    mint: self.mint.to_account_info(),
                    to: self.user_associated_token_account.to_account_info(),
                    authority: self.config.to_account_info(),
                },
                signer_seeds,
            ),
            amount,
            self.mint.decimals,
        );

        //update balance
        self.whitelist_entry.balance_amount = self
            .whitelist_entry
            .balance_amount
            .checked_sub(amount)
            .unwrap();
        Ok(())
    }
}
