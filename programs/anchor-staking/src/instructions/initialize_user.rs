use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    /// The user who wants to initialize their staking account
    /// Must be a signer to authorize the account creation
    #[account(mut)]
    pub user: Signer<'info>,

    /// User's staking account that will track their staking statistics
    /// PDA derived from user's public key to store points and staking amounts
    #[account(
        init,
        payer = user,
        seeds = [b"user", user.key().as_ref()],
        space = 8 + UserAccount::INIT_SPACE,
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    /// System program for account creation and rent management
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeUser<'info> {
    pub fn initialize_user(&mut self, bumps: &InitializeUserBumps) -> Result<()> {
        self.user_account.set_inner(UserAccount {
            points: 0,
            nft_staked_amount: 0,
            sol_staked_amount: 0,
            spl_staked_amount: 0,
            bump: bumps.user_account,
        });

        Ok(())
    }
}
