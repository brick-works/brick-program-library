pub mod utils;
pub mod error;
pub mod instructions;
pub mod state;
use anchor_lang::prelude::*;
use instructions::*;
use state::*;

declare_id!("brick5uEiJqSkfuAvMtKmq7kiuEVmbjVMiigyV51GRF");

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

    /// marketplace authority can edit fees and rewards configs
    pub fn edit_marketplace(
        ctx: Context<EditMarketplace>, 
        fees_config: Option<FeesConfig>,
        rewards_config: Option<RewardsConfig>,
    ) -> Result<()> {
        edit_marketplace::handler(ctx, fees_config, rewards_config)
    }

    /// seller can edit payment_mint and product_price
    pub fn edit_product(ctx: Context<EditProduct>, product_price: u64) -> Result<()> {
        edit_product::handler(ctx, product_price)
    }

    /// marketplace auth can create multiple bounty vaults (different mints)
    pub fn init_bounty(ctx: Context<InitBounty>) -> Result<()> {
        init_bounty::handler(ctx)
    }

    /// marketplace initialization:
    /// creates the access mint independently you want a permissionless marketplace or not
    pub fn init_marketplace(
        ctx: Context<InitMarketplace>,
        access_mint_bump: u8,
        fees_config: Option<FeesConfig>,
        rewards_config: Option<RewardsConfig>,
    ) -> Result<()> {
        init_marketplace::handler(
            ctx, 
            access_mint_bump,
            fees_config, 
            rewards_config,
        )
    }

    /// recommeded to read the Product state code to understand the meaning of this data structure 
    pub fn init_product(
        ctx: Context<InitProduct>,     
        id: [u8; 16],
        product_price: u64
    ) -> Result<()> {
        init_product::handler(ctx, id, product_price)
    }

    /// if a marketplace wants to change the reward mint, sellers and buyers have to create a new vault
    /// because there is only one PDA, reward is the authority of these vaults
    pub fn init_reward_vault(ctx: Context<InitRewardVault>) -> Result<()> {
        init_reward_vault::handler(ctx)
    }
    
    pub fn init_reward(ctx: Context<InitReward>) -> Result<()> {
        init_reward::handler(ctx)
    }
    
    /// manages the transfers (buyer -> seller and fees to marketplace authority)
    pub fn register_buy(ctx: Context<RegisterBuy>, amount: u32) -> Result<()> {
        register_buy::handler(ctx, amount)
    }

    /// when promotion is ended users can withdraw the funds stored in the vaults, managed by the reward PDA
    pub fn withdraw_reward(ctx: Context<WithdrawReward>) -> Result<()> {
        withdraw_reward::handler(ctx)
    }
}
