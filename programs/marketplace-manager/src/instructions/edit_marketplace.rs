use {
    crate::state::*,
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
        seeds = [Marketplace::get_seeds(&signer.key())],
        bump = marketplace.bumps.bump,
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
    if fees_config.is_some() {
        let fees: FeesConfig = fees_config.unwrap();
        Marketplace::validate_fees(&fees)?;
    }

    ctx.accounts.marketplace.fees_config = fees_config;
    ctx.accounts.marketplace.rewards_config = rewards_config;

    Ok(())
}