use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, TransferChecked};
use spl_token_2022::cmp_pubkeys;
use crate::{error::ErrorCode, event::BonusEvent};

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
    /// Fee reduction in absolute terms (ie fee 5% and reduction 2 value = total fee 3%)
    pub fee_reduction: Option<u16>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum PaymentFeePayer {
    Buyer,
    Seller,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardsConfig {
    /// The marketplace will only give rewards if the payment is made with this specific mint.
    pub reward_mint: Pubkey,
    /// The transaction volume percentage that the seller receives as a reward on a sale.
    /// A value of 250 corresponds to a reward of 2.5% of the transaction volume.
    /// A value of 0 indicates that there is no active rewards for the seller.
    pub seller_reward: u16,
    /// The transaction volume percentage that the buyer receives as a reward on a sale.
    pub buyer_reward: u16,
}

impl Marketplace {
    // optional properties needs one more byte (Borsh uses 1 extra byte to serialize options)
    pub const SIZE: usize = 8 
        + 32 //authority
        + 1 + 1 // bumps
        + 1 + 32 // access_mint
        + 1 + 2 + 1 + 1 + 32 + 1 + 2 // fees
        + 1 + 2 + 2 + 1 + 32; // rewards

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
            if fee_reduction > 10000 || fee_reduction > fees_config.fee {
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

    pub fn validate_vault(vault_owner: Pubkey, proof: Pubkey) -> bool {
        vault_owner == proof
    }

    pub fn is_rewards_active(&self, payment_mint: &Pubkey) -> bool {
        self.rewards_config
            .as_ref()
            .map_or(false, |rewards_config| cmp_pubkeys(&rewards_config.reward_mint, payment_mint))
    }    

    pub fn calculate_bonus(
        rewards_percentage: u16,
        product_price: u64,
    ) -> Result<u64> {
        let bonus = (rewards_percentage as u128)
            .checked_mul(product_price as u128)
            .ok_or(ErrorCode::NumericalOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::NumericalOverflow)? as u64;
        Ok(bonus)
    }
    
    pub fn transfer_bonus<'info>(
        &self,
        from: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        seller: AccountInfo<'info>,
        buyer: AccountInfo<'info>,
        authority: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        product_price: u64,
        decimals: u8,
    ) -> Result<()> {
        let marketplace_seeds = &[
            b"marketplace".as_ref(),
            self.authority.as_ref(),
            &[self.bumps.bump],
        ];

        let buyer_bonus = Marketplace::calculate_bonus(
            self.rewards_config.as_ref().unwrap().buyer_reward, 
            product_price
        )?;
        if buyer_bonus > 0 {
            transfer_checked(
                CpiContext::new_with_signer(
                    token_program.clone(),
                    TransferChecked {
                        from: from.clone(),
                        mint: mint.clone(),
                        to: buyer.clone(),
                        authority: authority.clone(),
                    },
                    &[&marketplace_seeds[..]],
                ),
                buyer_bonus,
                decimals,
            )?;

            emit!(BonusEvent {
                receiver: buyer.key().to_string(),
                mint: mint.key().to_string(),
                amount: buyer_bonus,
            });
        }

        let seller_bonus = Marketplace::calculate_bonus(
            self.rewards_config.as_ref().unwrap().seller_reward, 
            product_price
        )?;
        if seller_bonus > 0 {
            transfer_checked(
                CpiContext::new_with_signer(
                    token_program,
                    TransferChecked {
                        from,
                        mint: mint.clone(),
                        to: seller.clone(),
                        authority,
                    },
                    &[&marketplace_seeds[..]],
                ),
                seller_bonus,
                decimals,
            )?;

            emit!(BonusEvent {
                receiver: seller.key().to_string(),
                mint: mint.key().to_string(),
                amount: buyer_bonus,
            });
        }
    
        Ok(())
    }
}
