pub mod state;
pub mod utils;
pub mod error;
mod instructions;
use {
    anchor_lang::prelude::*,
    instructions::*,
};

declare_id!("7so9inMdB3rWrrQD4jq3s9Yq8AELRh8nsFKUTh3VWoe4");

#[program]
pub mod marketplace_manager {
    use super::*;

    /// airdrop a token that allows users to create products in a specific marketplace
    pub fn accept_access(ctx: Context<AcceptAccess>) -> Result<()> {
        accept_access::handler(ctx)
    }

    /// airdrop a token that allows users to create products in a specific marketplace
    pub fn airdrop_access(ctx: Context<AirdropAccess>) -> Result<()> {
        airdrop_access::handler(ctx)
    }

    /// seller can edit payment_mint and product_price
    pub fn edit_product(ctx: Context<EditProduct>, product_price: u64) -> Result<()> {
        edit_product::handler(ctx, product_price)
    }

    /// marketplace authority can edit fees and rewards configs
    pub fn edit_marketplace(ctx: Context<EditMarketplace>, params: EditMarketplaceParams) -> Result<()> {
        edit_marketplace::handler(ctx, params)
    }

    /// marketplace auth can create multiple bounty vaults (different mints)
    pub fn init_bounty(ctx: Context<InitBounty>) -> Result<()> {
        init_bounty::handler(ctx)
    }

    /// recommeded to read the Marketplace state code to understand the meaning of this data structure 
    pub fn init_marketplace(ctx: Context<InitMarketplace>, params: InitMarketplaceParams) -> Result<()> {
        init_marketplace::handler(ctx, params)
    }

    /// recommeded to read the Product state code to understand the meaning of this data structure 
    pub fn init_product_tree(ctx: Context<InitProductTree>, params: InitProductTreeParams) -> Result<()> {
        init_product_tree::handler(ctx, params)
    }

    /// recommeded to read the Product state code to understand the meaning of this data structure 
    pub fn init_product(ctx: Context<InitProduct>, params: InitProductParams) -> Result<()> {
        init_product::handler(ctx, params)
    }

    /// if a marketplace wants to change the reward mint, sellers and buyers have to create a new vault
    /// because there is only one PDA, reward is the authority of these vaults
    pub fn init_reward_vault(ctx: Context<InitRewardVault>) -> Result<()> {
        init_reward_vault::handler(ctx)
    }
    
    pub fn init_reward(ctx: Context<InitReward>) -> Result<()> {
        init_reward::handler(ctx)
    }
    
    pub fn register_buy_cnft(ctx: Context<RegisterBuyCnft>, params: RegisterBuyCnftParams) -> Result<()> {
        register_buy_cnft::handler(ctx, params)
    }

    /// manages the transfers (buyer -> seller and fees to marketplace authority) 
    /// and buyers receive a token as a proof of payment (each product has its own tokenc)
    pub fn register_buy_fungible(ctx: Context<RegisterBuyToken>, amount: u32) -> Result<()> {
        register_buy_fungible::handler(ctx, amount)
    }

    /// manages the transfers (buyer -> seller and fees to marketplace authority)
    /// uses payment pda to index transactions, but it does not initilize it
    pub fn register_buy(ctx: Context<RegisterBuy>, amount: u32) -> Result<()> {
        register_buy::handler(ctx, amount)
    }

    /// creates on chain request to get access to sell products in a specific marketplace
    pub fn request_access(ctx: Context<RequestAccess>) -> Result<()> {
        request_access::handler(ctx)
    }

    /// creates o new tree related to the product
    pub fn update_tree(ctx: Context<UpdateProductTree>, params: UpdateProductTreeParams) -> Result<()> {
        update_tree::handler(ctx, params)
    }
    
    /// when promotion is ended users can withdraw the funds stored in the vaults, managed by the reward PFA
    pub fn withdraw_reward(ctx: Context<WithdrawReward>) -> Result<()> {
        withdraw_reward::handler(ctx)
    }
}
