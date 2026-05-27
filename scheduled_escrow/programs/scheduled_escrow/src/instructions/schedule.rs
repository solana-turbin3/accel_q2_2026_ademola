use anchor_lang::{
    prelude::*,
    InstructionData,
};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use tuktuk_program::{
    compile_transaction,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
        types::TriggerV0,
    },
    types::QueueTaskArgsV0,
    TransactionSourceV0,
};
use crate::{Escrow, ESCROW_SEED};

#[derive(Accounts)]
pub struct Schedule<'info> {
     /// CHECK: escrow owner account
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        has_one = mint_a,
        has_one = maker,
        seeds = [ESCROW_SEED, maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Validated by tuktuk program
    #[account(mut)]
    pub task_queue: UncheckedAccount<'info>,
    /// CHECK: Validated by tuktuk program
    pub task_queue_authority: UncheckedAccount<'info>,
    /// CHECK: Validated by tuktuk program
    #[account(mut)]
    pub task: UncheckedAccount<'info>,
    /// CHECK: Queue authority account
    #[account(
           mut,
           seeds = [b"queue_authority"],
           bump
       )]
    pub queue_authority: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub tuktuk_program: Program<'info, Tuktuk>,
}

impl<'info> Schedule<'info> {

    pub fn schedule(&mut self, task_id: u16, bumps: ScheduleBumps) -> Result<()> {
        let (compiled_transaction, _) = compile_transaction(
            vec![Instruction {
                program_id: crate::ID,
                accounts: crate::__cpi_client_accounts_scheduled_refund::ScheduledRefund{
                    escrow: self.escrow.to_account_info(),
                    maker: self.maker.to_account_info(),
                    maker_ata: self.maker_ata.to_account_info(),
                    mint_a: self.mint_a.to_account_info(),
                    vault: self.vault.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                }
                .to_account_metas(None),
                data: crate::instruction::ScheduledRefund.data(),
            }],
            vec![],
        )?;

    queue_task_v0(
        CpiContext::new_with_signer(
            self.tuktuk_program.to_account_info(),
             QueueTaskV0 {
            payer: self.maker.to_account_info(),
            queue_authority: self.queue_authority.to_account_info(),
            task_queue: self.task_queue.to_account_info(),
            task_queue_authority: self.task_queue_authority.to_account_info(),
            task: self.task.to_account_info(),
            system_program: self.system_program.to_account_info(),
     },
     &[&["queue_authority".as_bytes(), &[bumps.queue_authority]]],
        ),
        QueueTaskArgsV0 {
            trigger: TriggerV0::Timestamp(self.escrow.expiry as i64),
            transaction: TransactionSourceV0::CompiledV0(compiled_transaction),
            crank_reward: Some(1000001),
            free_tasks: 0,
            id: task_id,
            description: "escrow".to_string(),
        },
    )?;

    Ok(())
}
}
