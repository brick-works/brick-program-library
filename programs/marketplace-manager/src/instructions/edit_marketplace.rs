use {
    crate::state::*,
    crate::utils::pda::get_marketplace_address,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
};
/// CHECK: this instruction only needs the marketplace authority == signer validation
#[derive(Accounts)]
pub struct EditMarketplace<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        address = get_marketplace_address(&signer.key()),
        constraint = signer.key() == marketplace.authority 
            @ ErrorCode::IncorrectAuthority,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
}

pub fn handler<'info>(
    ctx: Context<EditMarketplace>, 
    fees_config: Option<FeesConfig>,
    rewards_config: Option<RewardsConfig>,
) -> Result<()> {
    ctx.accounts.marketplace.fees_config = fees_config.clone();
    ctx.accounts.marketplace.rewards_config = rewards_config.clone();
    
    if fees_config.is_some() {
        ctx.accounts.marketplace.validate_fees()?;
    }
    if rewards_config.is_some() {
        ctx.accounts.marketplace.validate_rewards()?;
    }

    Ok(())
}