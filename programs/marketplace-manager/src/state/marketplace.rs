use std::mem::size_of;
use anchor_lang::prelude::*;
use crate::error::ErrorCode;

/// This account represents a marketplace with associated transaction fees and reward configurations.
/// The account is controlled by an authority that can modify the fee and reward configurations.
#[account]
pub struct Marketplace {
    /// The authorized entity that can modify this account data.
    pub authority: Pubkey,
    /// Seed bump parameters used for deterministic address derivation.
    pub bumps: MarketplaceBumps,
    /// If enabled, sellers need at least one token of this mint to list a product in your marketplace.
    pub access_mint: Option<Pubkey>,
    pub fees_config: Option<FeesConfig>,
    pub rewards_config: Option<RewardsConfig>,
}

/// Bump seed parameters used for deterministic address derivation.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MarketplaceBumps {
    pub bump: u8,
    pub access_mint_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FeesConfig {
    /// The transaction fee percentage levied by the app or marketplace.
    /// For example, a value of 250 corresponds to a fee of 2.5%.
    pub fee: u16,
    /// The entity that pays the transaction fees (either the buyer or the seller).
    pub fee_payer: PaymentFeePayer,
    /// This mint reduces the fee
    pub discount_mint: Option<Pubkey>,
    /// Fee reduction percentage applied if the seller chooses to receive a specific token as payment.
    pub fee_reduction: Option<u16>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum PaymentFeePayer {
    Buyer,
    Seller,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardsConfig {
    /// If set, the marketplace will only give rewards if the payment is made with this specific mint.
    /// To enable rewards irrespective of payment mint, set this value to default pubkey.
    pub reward_mint: Pubkey,
    /// The transaction volume percentage that the seller receives as a reward on a sale.
    /// A value of 250 corresponds to a reward of 2.5% of the transaction volume.
    /// A value of 0 indicates that there is no active rewards for the seller.
    pub seller_reward: u16,
    /// The transaction volume percentage that the buyer receives as a reward on a sale.
    pub buyer_reward: u16,
}

impl Marketplace {
    pub const SIZE: usize = 8 + size_of::<Marketplace>();

    pub fn initialize(
        &mut self,
        authority: Pubkey,
        bumps: MarketplaceBumps,
        access_mint: Option<Pubkey>,
        fees_config: Option<FeesConfig>,
        rewards_config: Option<RewardsConfig>,
    ) -> Result<()> {
        self.authority = authority;
        self.bumps = bumps;
        self.access_mint = access_mint;
        self.fees_config = fees_config;
        self.rewards_config = rewards_config;
        Ok(())
    }
    
    pub fn get_seeds(signer: &Pubkey) -> &'static [u8] {
        let seeds: &'static [u8] = &[b"marketplace".as_ref(), signer.as_ref()].concat();
        seeds
    }

    pub fn get_mint_seeds(marketplace: &Pubkey) -> &'static [u8] {
        let seeds: &'static [u8] = &[b"access_mint".as_ref(), marketplace.as_ref()].concat();
        seeds
    }

    pub fn get_vault_seeds(marketplace: &Pubkey, reward: &Pubkey) -> &'static [u8] {
        let seeds: &'static [u8] = &[b"bounty_vault".as_ref(), reward.as_ref()].concat();
        seeds
    }

    pub fn get_signer_seeds(
        signer: &Pubkey,
        bump: u8,
    ) -> &[&[&[u8]]]{
        &[&[
            b"marketplace".as_ref(),
            signer.as_ref(),
            &[bump],
        ]]
    }

    pub fn get_mint_signer_seeds(
        signer: &Pubkey,
        bump: u8,
    ) -> &[&[&[u8]]]{
        &[&[
            b"marketplace".as_ref(),
            signer.as_ref(),
            &[bump],
        ]]
    }

    pub fn validate_fees(fees_config: &FeesConfig) -> Result<()> {
        if fees_config.fee > 10000 {
            return Err(ErrorCode::IncorrectFee.into());
        }
    
        if let Some(fee_reduction) = fees_config.fee_reduction {
            if fee_reduction > 1000 {
                return Err(ErrorCode::IncorrectFee.into());
            }
        }
    
        Ok(())
    }

    pub fn validate_access(
        on_chain_mint: &Pubkey, 
        input_mint: &Pubkey,
        vault_owner: &Pubkey,
        signer: &Pubkey,
        vault_amount: u64,
    ) -> bool {
        if on_chain_mint != input_mint || vault_amount == 0 || vault_owner != signer {
            return false;
        }
        true
    }    
}
