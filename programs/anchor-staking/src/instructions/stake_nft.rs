use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        mpl_token_metadata::instructions::{
            FreezeDelegatedAccountCpi, FreezeDelegatedAccountCpiAccounts,
        },
        MasterEditionAccount, Metadata, MetadataAccount,
    },
    token::{approve, Approve, Mint, Token, TokenAccount},
};

use crate::error::ErrorCode;
use crate::{StakeAccount, StakeConfig, UserAccount};

/*
    check config -> validate user account and NFT ATA ->
    create Stake acc -> verify NFT belongs to collection ->
    delegate to stake account -> update stake and user account
*/
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct StakeNFT<'info> {
    /// The user who is staking their NFT
    /// Must be a signer to authorize the staking transaction
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

    /// Individual stake account for this specific NFT staking instance
    /// PDA derived from config, NFT mint, and unique seed to track this stake
    #[account(
        init,
        payer = user,
        seeds = [b"stake", config.key().as_ref(), nft_mint.key().as_ref(), seed.to_le_bytes().as_ref()],
        space = 8 + StakeAccount::INIT_SPACE,
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    /// The NFT mint being staked
    /// Must be mutable as it will be transferred to the staking vault
    pub nft_mint: Account<'info, Mint>,

    /// The collection mint that this NFT belongs to
    /// Used to verify the NFT is from the correct collection
    pub collection_mint: Account<'info, Mint>,

    /// User's Associated Token Account holding the NFT to be staked
    /// Must be mutable as the NFT will be transferred out of this account
    #[account(
        mut,
        associated_token::mint = nft_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_nft_ata: Account<'info, TokenAccount>,

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
    /// Created if it doesn't exist to hold earned rewards
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,

    /// NFT metadata account containing collection and attribute information
    /// PDA derived from metadata program and NFT mint to verify collection membership
    /// Constraint ensures the NFT belongs to the specified collection and is verified
    #[account(
        seeds = [b"metadata",metadata_program.key().as_ref(), nft_mint.key().as_ref()],
        bump,
        seeds::program = metadata_program.key()
    )]
    pub metadata: Account<'info, MetadataAccount>,

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

    /// System program for account creation and rent management
    pub system_program: Program<'info, System>,

    /// SPL Token program for token transfers and account management
    pub token_program: Program<'info, Token>,

    /// Metaplex Metadata program for NFT metadata verification
    pub metadata_program: Program<'info, Metadata>,

    /// Associated Token program for creating and managing ATAs
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> StakeNFT<'info> {
    pub fn stake_nft(
        &mut self,
        seed: u64,
        lock_period: i64,
        locked_stake: bool,
        bumps: &StakeNFTBumps,
    ) -> Result<()> {
        // Validate basic config parameters
        require!(
            lock_period >= self.config.min_freeze_period,
            ErrorCode::TooLowStakePeriod
        );

        // Validate NFT metadata and token correctness
        require!(
            self.metadata.mint == self.nft_mint.key(),
            ErrorCode::MetadataMintMismatch
        );
        let collection = self
            .metadata
            .collection
            .as_ref()
            .ok_or(ErrorCode::MissingCollection)?;
        require!(
            collection.key == self.collection_mint.key(),
            ErrorCode::WrongCollection
        );
        require!(collection.verified, ErrorCode::CollectionNotVerified);

        // Validate mint and ATA properties
        require!(self.nft_mint.decimals == 0, ErrorCode::InvalidMintDecimals);
        require!(self.nft_mint.supply == 1, ErrorCode::InvalidNftSupply);
        require!(
            self.user_nft_ata.owner == self.user.key(),
            ErrorCode::InvalidTokenOwner
        );
        require!(self.user_nft_ata.amount == 1, ErrorCode::InvalidTokenAmount);

        // Now it's safe to approve the delegate
        let cpi_program = self.token_program.to_account_info();
        let cpi_accounts = Approve {
            to: self.user_nft_ata.to_account_info(),
            delegate: self.stake_account.to_account_info(),
            authority: self.user.to_account_info(),
        };
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

        approve(cpi_context, 1)?;

        // Perform the freeze CPI
        let delegate = &self.stake_account.to_account_info();
        let token_account = &self.user_nft_ata.to_account_info();
        let edition = &self.master_edition.to_account_info();
        let mint = &self.nft_mint.to_account_info();
        let token_program = &self.token_program.to_account_info();
        let metadata_program = &self.metadata_program.to_account_info();

        let signers_seeds: &[&[&[u8]]] = &[&[
            b"stake",
            self.config.to_account_info().key.as_ref(),
            self.nft_mint.to_account_info().key.as_ref(),
            &seed.to_le_bytes()[..],
            &[bumps.stake_account],
        ]];

        FreezeDelegatedAccountCpi::new(
            metadata_program,
            FreezeDelegatedAccountCpiAccounts {
                delegate,
                token_account,
                edition,
                mint,
                token_program,
            },
        )
        .invoke_signed(signers_seeds)?;

        // Finally, update state
        self.stake_account.set_inner(StakeAccount {
            owner: self.user.key(),
            mint: self.nft_mint.key(),
            staked_amount: 1,
            staked_at: Clock::get()?.unix_timestamp,
            lock_period: lock_period,
            locked_stake: locked_stake,
            bump: bumps.stake_account,
            seed: seed,
        });

        if self.user_account.nft_staked_amount + 1 >= self.config.max_nft_stake {
            return Err(ErrorCode::ExceedsMaxNftStake.into());
        }

        self.user_account.nft_staked_amount = self
            .user_account
            .nft_staked_amount
            .checked_add(1)
            .ok_or_else(|| ErrorCode::Overflow)?;

        Ok(())
    }
}
