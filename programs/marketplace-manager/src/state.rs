use anchor_lang::prelude::*;

/// This account represents a marketplace with associated transaction fees and reward configurations.
/// The account is controlled by an authority that can modify the fee and reward configurations.
#[account]
pub struct Marketplace {
    /// The authorized entity that can modify this account data.
    pub authority: Pubkey,
    /// Token or indexing and access system work.
    pub token_config: TokenConfig,
    /// Set of permission configuration on a marketplace that can be modified by the authority.
    pub permission_config: PermissionConfig,
    /// Set of fee configuration that can be modified by the authority.
    pub fees_config: FeesConfig,
    /// Set of rewards configuration that can be modified by the authority.
    pub rewards_config: RewardsConfig,
    /// Seed bump parameters used for deterministic address derivation.
    pub bumps: MarketplaceBumps,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct TokenConfig {
    /// If true the fungible token can be transferable, ie: a user can sell it
    pub transferable: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct PermissionConfig {
    // If permissionless is false, sellers need to hold this token to create products on a specific marketplace.
    pub access_mint: Pubkey,
    /// True = permissionless marketplace, false = only wallets with a specific token can create products.
    pub permissionless: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FeesConfig {
    /// The transaction fee percentage levied by the app or marketplace.
    /// For example, a value of 250 corresponds to a fee of 2.5%.
    pub fee: u16,
    /// The entity that pays the transaction fees (either the buyer or the seller).
    pub fee_payer: PaymentFeePayer,
    /// This mint reduces the fee
    pub discount_mint: Pubkey,
    /// Fee reduction percentage applied if the seller chooses to receive a specific token as payment.
    pub fee_reduction: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum PaymentFeePayer {
    Buyer,
    Seller,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardsConfig {
    /// This flag enables or disables the reward system.
    /// When false, the reward system is inactive regardless of the reward_mint value.
    pub rewards_enabled: bool,
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

/// Bump seed parameters used for deterministic address derivation.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MarketplaceBumps {
    pub bump: u8,
    pub access_mint_bump: u8,
}

pub const MARKETPLACE_SIZE: usize = 8  // discriminator
    + 32  // authority
    // TokenConfig
    + 1   // transferable
    // PermissionConfig
    + 32  // access_mint
    + 1   // permissionless
    // FeesConfig
    + 2   // fee
    + 1   // fee_payer
    + 32  // discount_mint
    + 2   // fee_reduction
    // RewardsConfig
    + 1   // rewards_enabled
    + 32  // reward_mint
    + 2   // seller_reward
    + 2   // buyer_reward
    // MarketplaceBumps
    + 1   // bump
    + 1;  // access_mint_bump

/// This account works as an product administrator
#[account]
pub struct Product {
    /// The seller's public key, who owns the product.
    pub authority: Pubkey,
    pub id: [u8; 16],
    /// Two options:
    /// - Collection address
    /// - Mint (fungible) that represents the product. Owning this token implies having paid for the product.
    pub product_mint: Pubkey,
    /// Active merkle tree, a seller has a limited sells so when the tree is full it is needed to update this address
    /// can be null in case the product is created with create_product, to create a new one it is needed to call update_tree
    pub merkle_tree: Pubkey,
    /// Seller-defined product configurations.
    pub seller_config: SellerConfig,
    /// Seed bump parameters used for deterministic address derivation.
    pub bumps: ProductBumps,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SellerConfig {
    /// The token seller selects to receive as payment.
    pub payment_mint: Pubkey,
    /// The product price in terms of payment token/mint.
    pub product_price: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProductBumps {
    pub bump: u8,
    pub mint_bump: u8,
}

pub const PRODUCT_SIZE: usize = 8 // discriminator
    + 32 // authority
    + 16 // id
    + 32 // product_mint
    + 32 // merkle_tree
    // SellerConfig
    + 32 // payment_mint
    + 8  // product_price
    // ProductBumps
    + 1  // product_bump
    + 1; // mint_bump

#[account]
pub struct Reward {
    pub authority: Pubkey,
    pub bump: u8,
}

pub const REWARD_SIZE: usize = 8 // discriminator
    + 32  // authority
    + 1;  // bump

#[account]
pub struct Access {
    /// The user pubkey that request access to the marketplace.
    pub authority: Pubkey,
    pub bump: u8,
}

pub const ACCESS_SIZE: usize = 8 // discriminator
    + 32  // authority
    + 32  // marketplace
    + 1;  // bump
