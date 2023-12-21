use anchor_lang::prelude::*;
use std::mem::size_of;

#[account]
pub struct Reward {
    pub authority: Pubkey,
    pub bump: u8,
}

impl Reward {
    pub const SIZE: usize = 8 + size_of::<Reward>();

    pub fn initialize(&mut self, authority: Pubkey, bump: u8) -> Result<()> {
        self.authority = authority;
        self.bump = bump;
        Ok(())
    }
}