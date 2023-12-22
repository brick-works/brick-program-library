use anchor_lang::prelude::*;

/// This account represents a request to get the token neccessary to list products
#[account]
pub struct AccessRequest {
    /// The rent of this account will be resend to the payer when accepted
    pub payer: Pubkey,
}

impl AccessRequest {
    pub const SIZE: usize = 8 + 32;
}
