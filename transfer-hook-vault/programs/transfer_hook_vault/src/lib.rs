use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use spl_discriminator::discriminator::SplDiscriminate;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("w7jecRTVoH3FkVSDxuNHJN1PTFi83GieT138r1V3es7");

#[program]
pub mod transfer_hook_vault {
    use super::*;

    pub fn initialize(ctx: Context<InitializeVault>) -> Result<()> {
        ctx.accounts.initialize_vault(&ctx.bumps)
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_hook(amount)
    }

    pub fn whitelist_user(ctx: Context<WhitelistUser>, deposit_cap: u64) -> Result<()> {
        ctx.accounts.whitelist(deposit_cap, &ctx.bumps)
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        ctx.accounts.mint_tokens(amount)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn remove_whitelist_user(ctx: Context<RemoveFromWhitelist>) -> Result<()> {
        ctx.accounts.remove()
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }
}
