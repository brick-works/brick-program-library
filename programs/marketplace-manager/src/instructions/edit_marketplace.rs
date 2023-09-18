use {
    crate::state::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_spl::token_interface::Mint,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct EditMarketplaceParams {
    pub fee: u16,
    pub fee_reduction: u16,
    pub seller_reward: u16,
    pub buyer_reward: u16,
    pub use_cnfts: bool,
    pub deliver_token: bool,
    pub transferable: bool,
    pub chain_counter: bool,
    pub permissionless: bool,
    pub rewards_enabled: bool,
    pub fee_payer: PaymentFeePayer,
}

#[derive(Accounts)]
pub struct EditMarketplace<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"marketplace".as_ref(),
            signer.key().as_ref(),
        ],
        bump = marketplace.bumps.bump,
        constraint = signer.key() == marketplace.authority 
            @ ErrorCode::IncorrectAuthority,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    /// CHECK: no need to validate, marketplace auth is the unique wallet who can call this instruction
    pub reward_mint: UncheckedAccount<'info>,
    pub discount_mint: Box<InterfaceAccount<'info, Mint>>,
}

pub fn handler<'info>(
    ctx: Context<EditMarketplace>, 
    params: EditMarketplaceParams,
) -> Result<()> {
    if params.fee_reduction > 10000 || params.fee > 10000 || params.seller_reward > 10000 || params.buyer_reward > 10000 {
        return Err(ErrorCode::IncorrectFee.into());
    }

    (*ctx.accounts.marketplace).token_config = TokenConfig {
        use_cnfts: params.use_cnfts,
        deliver_token: params.deliver_token,
        transferable: params.transferable,
        chain_counter: params.chain_counter,
    };
    (*ctx.accounts.marketplace).permission_config = PermissionConfig {
        permissionless: params.permissionless,
        access_mint: ctx.accounts.marketplace.permission_config.access_mint,
    };
    (*ctx.accounts.marketplace).fees_config = FeesConfig {
        discount_mint: ctx.accounts.discount_mint.key(),
        fee: params.fee,
        fee_reduction: params.fee_reduction,
        fee_payer: params.fee_payer,
    };
    (*ctx.accounts.marketplace).rewards_config = RewardsConfig {
        reward_mint: ctx.accounts.reward_mint.key(),
        bounty_vaults: ctx.accounts.marketplace.rewards_config.bounty_vaults.clone(),
        seller_reward: params.seller_reward,
        buyer_reward: params.buyer_reward,
        rewards_enabled: params.rewards_enabled,
    };
    
    Ok(())
}