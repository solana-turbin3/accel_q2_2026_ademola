use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;


pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("G7hNNNmi9HZq8HXBiR1n4jG3Qkd8F3JhJG34wsothuKu");

#[program]
pub mod scheduled_escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, deposit: u64, expiry_duration: u64, receive: u64) -> Result<()> {
        ctx.accounts.init_escrow(seed, receive, expiry_duration, &ctx.bumps)?;
        ctx.accounts.deposit(deposit)?;
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.deposit()?;
        ctx.accounts.withdraw_and_close_vault()?;
        Ok(())
    }

    pub fn schedule(ctx: Context<Schedule>, task_id: u16) -> Result<()> {
        ctx.accounts.schedule(task_id, ctx.bumps)?;
        Ok(())
    }

    pub fn scheduled_refund(ctx: Context<ScheduledRefund>) -> Result<()> {
        ctx.accounts.scheduled_refund()?;
        Ok(())
    }

    pub fn manual_refund(ctx: Context<Refund>) -> Result<()> {
        ctx.accounts.refund_and_close_vault()
    }
}
