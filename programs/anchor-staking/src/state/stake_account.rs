use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeAccount {
    pub owner: Pubkey,        // who staked (the user)
    pub mint: Pubkey,         // NFT or token mint being staked
    pub staked_amount: u64,   // how much (for fungible or numeric systems)
    pub staked_at: i64,       // timestamp when staking started
    pub lock_period: i64,     // how long it’s locked (in seconds)
    pub locked_stakers: bool, // true if currently locked
    pub bump: u8,             // PDA bump
    pub seed: u64,            // unique seed to derive the stake PDA
}
