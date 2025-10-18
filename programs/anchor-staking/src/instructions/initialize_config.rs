use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

use crate::{error::ErrorCode, StakeConfig, ADMIN};

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    /// The admin/authority who can initialize the staking configuration
    /// Must match the predefined ADMIN public key for security
    #[account(
        mut,
        address = ADMIN @ ErrorCode::InvalidAdmin
    )]
    pub admin: Signer<'info>,

    /// Global staking configuration account that stores system parameters
    /// PDA that will hold reward rates, limits, and other staking settings
    #[account(
        init,
        payer = admin,
        space = 8 + StakeConfig::INIT_SPACE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, StakeConfig>,

    /// Reward token mint that will be used to distribute staking rewards
    /// PDA controlled by the config account with 6 decimal places
    #[account(
        init_if_needed,
        payer = admin,
        seeds = [b"reward", config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config 
    )]
    pub reward_mint: Account<'info, Mint>,

    /// System program for account creation and rent management
    pub system_program: Program<'info, System>,
    
    /// SPL Token program for creating the reward mint
    pub token_program: Program<'info, Token>
}

impl<'info> InitializeConfig<'info> {
    pub fn initialize_config(
        &mut self, 
        points_per_nft_stake: u8, 
        points_per_sol_stake: u8, 
        points_per_spl_stake: u8,
        min_freeze_period: i64,
        max_nft_stake: u8,
        apr: u16,
        bumps: &InitializeConfigBumps
    ) -> Result<()> {
        self.config.set_inner(StakeConfig { 
            points_per_nft_stake, 
            points_per_sol_stake, 
            points_per_spl_stake, 
            max_nft_stake,
            min_freeze_period, 
            apr, 
            reward_mint: self.reward_mint.key(), 
            authority: self.admin.key(), 
            rewards_bump: bumps.reward_mint, 
            bump: bumps.config 
        });
        
        Ok(())
    }
}
