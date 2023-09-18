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

/// Marketplace permission configs, gives flexibility to this program, marketplace auth decides.
/// All properties = false, buyer can call only register_buy instruction
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct TokenConfig {
    /// If true seller inits the tree and buyers receives a cnft as a proof of payment
    /// ie: seller calls init_product_tree to list a product and buyer calls register_buy_cnft
    pub use_cnfts: bool,
    /// If true when someone buys a product, he receives a fungible token as proof of payment
    /// false, payment account (just a counter) its created to keep track of amount of units bought
    pub deliver_token: bool,
    /// If true the fungible token can be transferable, ie: a user can sell it
    pub transferable: bool,
    /// If true payment account is created (pda used to index ALL transactions), init that account means having
    /// a counter of the times a user has bought a specific product (should be cheaper vs using a token)
    pub chain_counter: bool,
}

/// Marketplace permission configs, gives flexibility to this program, marketplace auth decides.
#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct PermissionConfig {
    // If permissionless is false, sellers need to hold this token to create products on a specific marketplace.
    pub access_mint: Pubkey,
    /// True = permissionless marketplace, false = only wallets with a specific token can create products.
    pub permissionless: bool,
}

/// Marketplace fees related to transactions.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FeesConfig {
    /// Marketplaces can set a mint. Sellers opting to receive this mint as payment
    /// will have their marketplace fee reduced. This could be a governance token, for instance.
    pub discount_mint: Pubkey,
    /// The transaction fee percentage levied by the app or marketplace.
    /// For example, a value of 250 corresponds to a fee of 2.5%.
    pub fee: u16,
    /// Fee reduction percentage applied if the seller chooses to receive a specific token as payment.
    pub fee_reduction: u16,
    /// The entity that pays the transaction fees (either the buyer or the seller).
    pub fee_payer: PaymentFeePayer,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum PaymentFeePayer {
    Buyer,
    Seller,
}

/// Rewards configuration associated with sales.
/// 1. marketplace auth init_market (with one bounty_vault)
/// 2. if a marketplace wants to change the reward mint, needs to call init_bounty_vault with that mint
/// 3. marketplace auth transfers manually the bounty tokens (can be with 1/2 in the same transaction)
/// 4. user sells / buys during promo in the marketplace, sends some tokens to the reward_vault of both 
/// seller and buyer these vaults are controlled by marketplace pda
/// 5. when promotion is ended the user can withdraw the rewards
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RewardsConfig {
    /// If set, the marketplace will only give rewards if the payment is made with this specific mint.
    /// To enable rewards irrespective of payment mint, set this value to pda(b"null", this program).
    pub reward_mint: Pubkey,
    /// Vaults where reward tokens are stored.
    /// These vaults are managed by this marketplace PDA.
    /// Tokens used during the promotional period should be deposited here.
    pub bounty_vaults: Vec<Pubkey>,
    /// The transaction volume percentage that the seller receives as a reward on a sale.
    /// A value of 250 corresponds to a reward of 2.5% of the transaction volume.
    /// A value of 0 indicates that there is no active rewards for the seller.
    pub seller_reward: u16,
    /// The transaction volume percentage that the buyer receives as a reward on a sale.
    pub buyer_reward: u16,
    /// This flag enables or disables the reward system.
    /// When false, the reward system is inactive regardless of the reward_mint value.
    pub rewards_enabled: bool,
}

/// Bump seed parameters used for deterministic address derivation.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MarketplaceBumps {
    pub bump: u8,
    pub vault_bumps: Vec<u8>,
    pub access_mint_bump: u8,
}

pub const VAULT_COUNT: usize = 5;
pub const MARKETPLACE_SIZE: usize = 8  // discriminator
    + 32  // authority
    // TokenConfig
    + 1   // deliver_token
    + 1   // transferable
    + 1   // chain_counter
    // PermissionConfig
    + 32  // access_mint
    + 1   // permissionless
    // FeesConfig
    + 32  // discount_mint
    + 2   // fee
    + 2   // fee_reduction
    + 1   // fee_payer
    // RewardsConfig
    + 32  // reward_mint
    + 32  // bounty_vaults
    * VAULT_COUNT
    + 2   // seller_reward
    + 2   // buyer_reward
    + 1   // rewards_enabled
    // MarketplaceBumps
    + 1   // bump
    + 1   // vault_bumps
    * VAULT_COUNT
    + 1;  // access_mint_bump

/// This account works as an product administrator
#[account]
pub struct Product {
    /// The seller's public key, who owns the product.
    pub authority: Pubkey,
    /// Off chain identifier of the product, split across two arrays due to a limit on
    /// the maximum size of a seed component and with the goal of use 64 byte id
    pub first_id: [u8; 32], 
    pub second_id: [u8; 32],
    // Where the product come from
    pub marketplace: Pubkey,
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
    + 32 // first_id
    + 32 // second_id
    + 32 // marketplace
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
    /// The public key of the account having authority over the reward PDA.
    pub authority: Pubkey,
    /// The marketplace address, stored to derive reward pda in the context.
    pub marketplace: Pubkey,
    /// Vault where tokens are stored until promotion is ended, then the user can withdraw.
    /// reward_mint is stored in marketplace account.
    /// It is allowed to create 5 vaults with different mints. In init_reward one is created, 
    /// if you want to change the mint reward for your users you need to makes user sign a init_bounty ix
    pub reward_vaults: Vec<Pubkey>,
    /// Seed bump parameter used for deterministic address derivation in case of the Reward account.
    pub bumps: RewardBumps,
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone)]
pub struct RewardBumps {
    pub bump: u8,
    pub vault_bumps: Vec<u8>,
}

pub const REWARD_SIZE: usize = 8 // discriminator
    + 32  // authority
    + 32  // marketplace
    + 32  // reward_vaults
    * VAULT_COUNT
    + 1   // bump
    + 1   // vault_bumps
    * VAULT_COUNT;


#[account]
pub struct Access {
    /// The user pubkey that request access to the marketplace.
    pub authority: Pubkey,
    /// The marketplace address, stored to derive access pda in the context.
    pub marketplace: Pubkey,
    pub bump: u8,
}

pub const ACCESS_SIZE: usize = 8 // discriminator
    + 32  // authority
    + 32  // marketplace
    + 1;  // bump
    
/// its a pda from signer, marketplace and product can only be 
/// created/modified in register_buy that requieres transfers
#[account]
pub struct Payment {
    pub units: u32,
    pub bump: u8,
}

pub const PAYMENT_SIZE: usize = 8 + 4 + 1;
