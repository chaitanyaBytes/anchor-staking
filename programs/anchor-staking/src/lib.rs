pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Avo64dZaN1yYtya9RLifN2MvPYfj17FiqyEzXpqJhSm1");

#[program]
pub mod anchor_staking {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize_config::handler(ctx)
    }
}
