use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct StakeConfig {
    pub points_per_nft_stake: u8, // base points per NFT
    pub points_per_sol_stake: u8, // points multiplier per SOL staked
    pub points_per_spl_stake: u8, // points multiplier per SPL staked
    pub max_nft_stake: u8,        // max NFTs a user can stake
    pub min_freeze_period: u32,   // seconds or epochs?
    pub apr: u16,                 // in basis points (e.g. 700 = 7.00%)
    pub reward_mint: Pubkey,      // SPL token used for rewards
    pub authority: Pubkey,        // admin/owner who can update config
    pub rewards_bump: u8,
    pub bump: u8,
}
