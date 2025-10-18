use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        mpl_token_metadata::instructions::{
            ThawDelegatedAccountCpi, ThawDelegatedAccountCpiAccounts,
        },
        MasterEditionAccount, Metadata,
    },
    token::{revoke, Mint, Revoke, Token, TokenAccount},
};

use crate::{error::ErrorCode, StakeAccount, StakeConfig, UserAccount};

/*
    Check if user can unstake -> verify lock period has passed ->
    transfer NFT back to user -> Give rewards to user ->
    close stake account -> update user stats
*/
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct UnstakeNFT<'info> {
    /// The user who is unstaking their NFT
    /// Must be a signer to authorize the unstaking transaction
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

    /// Individual stake account for the specific NFT being unstaked
    /// PDA derived from config, NFT mint, and unique seed to track this stake
    #[account(
        mut,
        close = user,
        seeds = [b"stake", config.key().as_ref(), nft_mint.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == user.key() @ ErrorCode::Unauthorized
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// The NFT mint that was staked and is now being unstaked
    /// Used to identify the specific stake and verify ownership
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    /// User's Associated Token Account that will receive the unstaked NFT
    /// Must be mutable as the NFT will be transferred back to this account
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_nft_ata: Account<'info, TokenAccount>,

    /// The staking vault's Associated Token Account holding the staked NFT
    /// Must be mutable as the NFT will be transferred out of this account
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = config,
        associated_token::token_program = token_program
    )]
    pub vault_nft_ata: Account<'info, TokenAccount>,

    /// The reward token mint used for distributing staking rewards
    /// PDA controlled by the config account for minting reward tokens
    #[account(
        mut,
        seeds = [b"reward", config.key().as_ref()],
        bump = config.rewards_bump,
        mint::authority = config
    )]
    pub reward_mint: Account<'info, Mint>,

    /// User's Associated Token Account for receiving any final reward tokens
    /// Must exist to hold any remaining rewards before unstaking
    #[account(
        mut,
        associated_token::mint = reward_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    /// Master edition account for the NFT being staked
    /// PDA derived from metadata program, NFT mint, and "edition" seed
    /// Required to verify the NFT is a legitimate NFT with proper edition data
    #[account(
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            nft_mint.key().as_ref(),
            b"edition"
        ],
        bump,
        seeds::program = metadata_program.key(),
    )]
    pub master_edition: Account<'info, MasterEditionAccount>,

    /// System program for account management and closure
    pub system_program: Program<'info, System>,

    /// SPL Token program for token transfers and account management
    pub token_program: Program<'info, Token>,

    /// Associated Token program for managing ATAs
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Metaplex Metadata program for NFT metadata verification
    pub metadata_program: Program<'info, Metadata>,
}

impl<'info> UnstakeNFT<'info> {
    pub fn unstake(&mut self) -> Result<()> {
        let staked_at = self.stake_account.staked_at;
        let current_time = Clock::get()?.unix_timestamp;

        let time_elapsed = current_time.checked_sub(staked_at).unwrap();

        require!(
            time_elapsed >= self.stake_account.lock_period,
            ErrorCode::FreezePeriodNotPassed
        );

        let seeds = &[
            b"stake",
            self.config.to_account_info().key.as_ref(),
            self.nft_mint.to_account_info().key.as_ref(),
            &self.stake_account.seed.to_le_bytes()[..],
            &[self.stake_account.bump],
        ];

        let signers_seeds: &[&[&[u8]]] = &[&seeds[..]];

        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.user_nft_ata.to_account_info();
        let edition = &self.master_edition.to_account_info();
        let mint = &self.nft_mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        ThawDelegatedAccountCpi::new(
            metadata_program,
            ThawDelegatedAccountCpiAccounts {
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            },
        )
        .invoke_signed(signers_seeds)?;

        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Revoke {
            source: self.user_nft_ata.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        revoke(cpi_context)?;

        Ok(())
    }
}
