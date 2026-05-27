use anchor_lang::prelude::*;


use anchor_spl::{ token_interface::{
    CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked, close_account, transfer_checked
}};

use crate::{state::Escrow, ESCROW_SEED, error::EscrowError};

#[derive(Accounts)]
pub struct ScheduledRefund<'info>{
    pub mint_a: InterfaceAccount<'info, Mint>,
    /// CHECK: escrow owner account
    #[account(mut)]
    pub maker: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [ESCROW_SEED, maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info>ScheduledRefund<'info>{
    pub fn scheduled_refund(&mut self) -> Result<()>{

        //checking expiry
        let now = Clock::get()?.unix_timestamp as u64;
        require!(now >= self.escrow.expiry, EscrowError::EscrowNotExpired);


        let signer_seeds: [&[&[u8]]; 1] = [&[
            ESCROW_SEED,
            self.maker.key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let cpi_accounts = TransferChecked {
            authority: self.escrow.to_account_info(),
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_ata.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(), 
            cpi_accounts,
            &signer_seeds
        );

        transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;

        close_account(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(), 
                CloseAccount { 
                    account: self.vault.to_account_info(),
                    destination: self.maker.to_account_info(),
                    authority: self.escrow.to_account_info() 
                }, 
                &signer_seeds
            )
        )?;


        Ok(())
    }
}
