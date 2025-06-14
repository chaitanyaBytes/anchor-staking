use anchor_lang::prelude::*;

use crate::StakeConfig;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        seeds = [b"config"],
        bump, 
        space = 8 + StakeConfig::INIT_SAPCE
    )]
    pub config: Account<'info, StakeConfig>,

    #[account(
        init, 
        payer = admin,
        seeds = [b"rewards", config.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = config,
    )]
    pub reward_mint: Account<'info, Mint>,

    pub token_program: 
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    msg!("Greetings from: {:?}", ctx.program_id);
    Ok(())
}
