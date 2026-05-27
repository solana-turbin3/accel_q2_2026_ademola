#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;
use ephemeral_rollups_sdk::anchor::ephemeral;

mod state;
mod utils;
mod instructions;

use instructions::*;

declare_id!("4bCtbRYWtdcWX8J3W99P3nVR6r2JFsZgDgB5sejKN4Ho");

#[ephemeral]
#[program]
pub mod er_state_account {

    use super::*;

    pub fn initialize(ctx: Context<InitUser>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;
        
        Ok(())
    }

    pub fn update(ctx: Context<UpdateUser>, new_data: u64) -> Result<()> {
        ctx.accounts.update(new_data)?;
        
        Ok(())
    }

    pub fn update_commit(ctx: Context<UpdateCommit>, new_data: u64) -> Result<()> {
        ctx.accounts.update_commit(new_data)?;
        
        Ok(())
    }

    pub fn delegate(ctx: Context<Delegate>) -> Result<()> {
        ctx.accounts.delegate()?;
        
        Ok(())
    }

    pub fn undelegate(ctx: Context<Undelegate>) -> Result<()> {
        ctx.accounts.undelegate()?;
        
        Ok(())
    }

    //VRF OUTSIDE ER
    pub fn request_randomness(ctx: Context<RequestRandomness>, client_seed: u8) -> Result<()> {
        ctx.accounts.request_randomness(client_seed)?;
        Ok(())
    }

    //VRF CONSUMPTION OUTSIDE ER
    pub fn consume_randomness(ctx: Context<ConsumeRandomness>, randomness: [u8; 32]) -> Result<()> {
        ctx.accounts.consume_randomness(randomness)?;
        Ok(())
    }

   // VRF REQUEST AND CONSUMPTION INSIDE ER
   pub fn request_randomness_commit(ctx: Context<RequestRandomnessCommit>, client_seed: u8) -> Result<()> {
        ctx.accounts.request_randomness_commit(client_seed)?;
        Ok(())
    }

    pub fn consume_randomness_commit(ctx: Context<ConsumeRandomnessCommit>, randomness: [u8; 32]) -> Result<()> {
        ctx.accounts.consume_randomness_commit(randomness)?;
        Ok(())
    }

    pub fn randomize(ctx: Context<Randomize>) -> Result<()> {
        ctx.accounts.randomize()?;

        Ok(())
    }

    pub fn randomize_commit(ctx: Context<RandomizeCommit>) -> Result<()> {
        ctx.accounts.randomize_commit()?;

        Ok(())
    }

    pub fn close(ctx: Context<CloseUser>) -> Result<()> {
        ctx.accounts.close()?;
        
        Ok(())
    }
}

