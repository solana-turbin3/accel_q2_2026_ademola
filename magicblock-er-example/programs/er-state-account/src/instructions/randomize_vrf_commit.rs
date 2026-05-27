use anchor_lang::prelude::*;
use crate::state::UserAccount;
use crate::ID;

use ephemeral_vrf_sdk::anchor::vrf;
use ephemeral_vrf_sdk::instructions::{create_request_randomness_ix, RequestRandomnessParams};
use ephemeral_vrf_sdk::types::SerializableAccountMeta;


#[commit]
#[derive(Accounts)]
pub struct ConsumeRandomnessCommit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    ///CHECK: VRF program identity signer
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

#[vrf]
#[derive(Accounts)]
pub struct RequestRandomnessCommit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,
    /// CHECK: The oracle queue
    #[account(mut, address = ephemeral_vrf_sdk::consts::DEFAULT_QUEUE)]
    pub oracle_queue: AccountInfo<'info>,
}

impl<'info> RequestRandomnessCommit<'info> {
    pub fn request_randomness_commit(&mut self, client_seed: u8) -> Result<()> {
        let ix = create_request_randomness_ix(RequestRandomnessParams { 
            payer: self.user.key(), 
            oracle_queue: self.oracle_queue.key(),
            callback_program_id: ID,
            callback_discriminator: crate::instruction::ConsumeRandomnessCommit::DISCRIMINATOR.to_vec(),
            caller_seed: [client_seed; 32],
            accounts_metas: Some(vec![SerializableAccountMeta {
                pubkey: self.user_account.key(),
                is_signer: false,
                is_writable: true,
            }]),
            ..Default::default()
        });

        self.invoke_signed_vrf(&self.user.to_account_info(), &ix)?;
        Ok(())
}
}