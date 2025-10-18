use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Avo64dZaN1yYtya9RLifN2MvPYfj17FiqyEzXpqJhSm1");

#[program]
pub mod anchor_staking {
    use super::*;
}
