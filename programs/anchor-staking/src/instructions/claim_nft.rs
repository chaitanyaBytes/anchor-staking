use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{error::ErrorCode, StakeAccount, StakeConfig, UserAccount};

/*
    Check if user can claim rewards -> calculate rewards based on staking time ->
    mint reward tokens -> update user points -> update stake account
*/
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct ClaimNFT<'info> {
    /// The user who is claiming rewards from their staked NFT
    /// Must be a signer to authorize the claim transaction
    #[account(mut)]
    pub user: Signer<'info>,

    /// User's staking account that tracks their overall staking statistics
    /// PDA derived from user's public key to store points and staking amounts
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump
    )]
    pub user_account: Account<'info, UserAccount>,

    /// Global staking configuration containing reward rates and limits
    /// PDA that stores system-wide staking parameters and reward settings
    #[account(
        seeds = [b"config"],
        bump = config.bump
    )]
    pub config: Account<'info, StakeConfig>,

    /// Individual stake account for the specific NFT being claimed
    /// PDA derived from config, NFT mint, and unique seed to track this stake
    #[account(
        mut,
        seeds = [b"stake", config.key().as_ref(), nft_mint.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == user.key() @ ErrorCode::Unauthorized
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// The NFT mint that was staked and is now claiming rewards for
    /// Used to identify the specific stake and verify ownership
    pub nft_mint: Account<'info, Mint>,

    /// The reward token mint used for distributing staking rewards
    /// PDA controlled by the config account for minting reward tokens
    #[account(
        mut,
        seeds = [b"reward", config.key().as_ref()],
        bump = config.rewards_bump,
        mint::authority = config
    )]
    pub reward_mint: Account<'info, Mint>,

    /// User's Associated Token Account for receiving reward tokens
    /// Must exist to hold the claimed rewards
    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    /// System program for account management
    pub system_program: Program<'info, System>,

    /// SPL Token program for token transfers and minting
    pub token_program: Program<'info, Token>,

    /// Associated Token program for managing ATAs
    pub associated_token_program: Program<'info, AssociatedToken>,
}
