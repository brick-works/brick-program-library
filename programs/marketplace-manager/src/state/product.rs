use anchor_lang::prelude::*;
use std::mem::size_of;

/// This account works as an product administrator
#[account]
pub struct Product {
    /// The seller's public key, who owns the product.
    pub authority: Pubkey,
    pub id: [u8; 16],
    /// Seller-defined product configurations.
    pub seller_config: SellerConfig,
    /// Seed bump used for deterministic address derivation.
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SellerConfig {
    /// The token seller selects to receive as payment.
    pub payment_mint: Pubkey,
    /// The product price in terms of payment token/mint.
    pub product_price: u64,
}

impl Product {
    pub const SIZE: usize = 8 + size_of::<Product>();

    pub fn initialize(
        &mut self, 
        authority: Pubkey, 
        id: [u8; 16],
        seller_config: SellerConfig, 
        bump: u8
    ) -> Result<()> {
        self.authority = authority;
        self.id = id;
        self.bump = bump;
        Ok(())
    }

    pub fn get_seeds(authority: &Pubkey) -> &'static [u8] {
        let seeds: &'static [u8] = &[b"product".as_ref(), authority.as_ref()].concat();
        seeds
    }
}