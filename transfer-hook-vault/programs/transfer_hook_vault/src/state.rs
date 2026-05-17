use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub enum EntryStatus {
    Suspended,
    Active,
}

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub bump: u8,
    pub mint: Pubkey,
    pub vault: Pubkey,
    pub suspended: bool,
}

#[account]
#[derive(InitSpace)]
pub struct WhitelistEntry {
    pub balance_amount: u64,
    pub deposit_cap: u64,
    pub bump: u8,
    pub status: EntryStatus,
}
