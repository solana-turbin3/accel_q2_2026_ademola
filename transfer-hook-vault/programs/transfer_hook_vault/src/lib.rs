pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;
use spl_discriminator::discriminator::SplDiscriminate;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("13HzQvReEjdNXroQvg2iHEkpcBGL3vSJA1sX5mrz6GBz");

#[program]
pub mod transfer_hook_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize()
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_hook(amount)
    }

    pub fn whitelist_user(ctx: Context<WhitelistUser>, deposit_cap: u64) -> Result<()> {
        ctx.accounts.whitelist(deposit_cap, &ctx.bumps)
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
