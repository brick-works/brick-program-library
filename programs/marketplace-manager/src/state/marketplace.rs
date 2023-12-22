use std::mem::size_of;
use anchor_lang::prelude::*;
use spl_token_2022::cmp_pubkeys;
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
    /// This mint reduces the fee & can make the rewards to be enforced to a unique mint.
    /// so you can incentive transactions with your own token
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
    /// The transaction volume percentage that the seller receives as a reward on a sale.
    /// A value of 250 corresponds to a reward of 2.5% of the transaction volume.
    /// A value of 0 indicates that there is no active rewards for the seller.
    pub seller_reward: u16,
    /// The transaction volume percentage that the buyer receives as a reward on a sale.
    pub buyer_reward: u16,
    /// If set, the marketplace will only give rewards if the payment is made with this specific mint.
    /// To enable rewards irrespective of payment mint, set this value to default pubkey.
    pub reward_mint: Option<Pubkey>,
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

    pub fn validate_fees(&mut self) -> Result<()> {
        let fees_config = self.fees_config.as_ref().unwrap();
        if fees_config.fee > 10000 {
            return Err(ErrorCode::InvalidFeeValue.into());
        }
    
        if let Some(fee_reduction) = fees_config.fee_reduction {
            if fee_reduction > 1000 {
                return Err(ErrorCode::InvalidFeeReductionValue.into());
            }
        }
    
        Ok(())
    }
    
    pub fn validate_rewards(&mut self) -> Result<()> {
        let rewards_config = self.rewards_config.as_ref().unwrap();
        if rewards_config.seller_reward > 10000 || rewards_config.buyer_reward > 10000 {
            return Err(ErrorCode::InvalidRewardValue.into());
        }
    
        Ok(())
    }

    pub fn is_rewards_active(&self, payment_mint: &Pubkey) -> bool {
        self.rewards_config
            .as_ref()
            .map_or(false, |config| {
                config.reward_mint
                    .map_or(true, |enforced_mint| cmp_pubkeys(&enforced_mint, payment_mint))
            })
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
