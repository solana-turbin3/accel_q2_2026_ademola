use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"vault_config", admin.key().as_ref()],
        space = 8 + Config::INIT_SPACE,
        bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 9,
        mint::authority = config,
        mint::token_program = token_program,
        extensions::transfer_hook::authority = config,
        extensions::transfer_hook::program_id = crate::ID,
        extensions::permanent_delegate::delegate = config,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = admin,
        associated_token::mint = mint,
        associated_token::authority = config,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ExtraAccountMetaList, seeds must match transfer hook handler
    #[account(
        init,
        payer = admin,
        space = ExtraAccountMetaList::size_of(
            InitializeVault::extra_account_metas().len()
        ).unwrap(),
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: AccountInfo<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeVault<'info> {
    pub fn extra_account_metas() -> Vec<ExtraAccountMeta> {
        vec![
            // Extra meta 0: this program itself (hook_program).
            // LiteSVM needs the callee's AccountInfo in cpi_account_infos to execute
            // the hook CPI; registering it here causes invoke_execute to include it.
            ExtraAccountMeta::new_with_pubkey(&crate::ID, false, false).unwrap(),
            // Extra meta 1: whitelist_entry PDA for the transfer source owner (index 3)
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: b"whitelist_entry".to_vec(),
                    },
                    Seed::AccountKey { index: 3 },
                ],
                false,
                false,
            )
            .unwrap(),
        ]
    }

    pub fn initialize_vault(&mut self, bumps: &InitializeVaultBumps) -> Result<()> {
        self.config.set_inner(Config {
            admin: self.admin.key(),
            mint: self.mint.key(),
            vault: self.vault.key(),
            bump: bumps.config,
            suspended: false,
        });

        let metas = Self::extra_account_metas();
        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut self.extra_account_meta_list.try_borrow_mut_data()?,
            &metas,
        )?;

        Ok(())
    }
}
