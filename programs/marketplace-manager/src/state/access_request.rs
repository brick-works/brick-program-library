use std::mem::size_of;
use anchor_lang::prelude::*;

/// This account represents a request to get the token neccessary to list products
#[account]
pub struct AccessRequest {
    /// The authorized entity that can modify this account data.
    pub authority: Pubkey,
    pub bump: u8,
}

impl AccessRequest {
    pub const SIZE: usize = 8 + size_of::<AccessRequest>();

    pub fn initialize(
        &mut self,
        authority: Pubkey,
        bump: u8,
    ) -> Result<()> {
        self.authority = authority;
        self.bump = bump;
        Ok(())
    }

    pub fn get_seeds(authority: &Pubkey, marketplace: &Pubkey) -> &'static [u8] {
        let seeds: &'static [u8] = &[b"request".as_ref(), authority.as_ref(), marketplace.as_ref()].concat();
        seeds
    }  
}
