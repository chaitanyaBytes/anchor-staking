use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub points: u32,            // total points earned
    pub nft_staked_amount: u8,  // number of NFTs staked
    pub sol_staked_amount: u64, // total SOL staked
    pub spl_staked_amount: u64, // total SPL staked
    pub bump: u8,
}
