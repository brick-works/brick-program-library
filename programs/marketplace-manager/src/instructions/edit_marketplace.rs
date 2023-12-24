use {
    crate::state::*,
    crate::utils::pda::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_spl::token_interface::Mint
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
    #[account(
        mut,
        address = get_access_mint_address(&marketplace.key()),
    )]    
    pub access_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
}

/// if arguments are null are also updated in the account like that
/// when are some the data is validated (access mint could be another token)
pub fn handler<'info>(
    ctx: Context<EditMarketplace>,
    fees_config: Option<FeesConfig>,
    rewards_config: Option<RewardsConfig>,
) -> Result<()> {
    ctx.accounts.marketplace.fees_config = fees_config.clone();
    if ctx.accounts.marketplace.fees_config.is_some() {
        ctx.accounts.marketplace.validate_fees()?;
    }

    ctx.accounts.marketplace.rewards_config = rewards_config.clone();
    if ctx.accounts.marketplace.rewards_config.is_some() {
        ctx.accounts.marketplace.validate_rewards()?;
    }

    if ctx.accounts.access_mint.is_some() {
        ctx.accounts.marketplace.access_mint = Some(ctx.accounts.access_mint.as_ref().unwrap().key());
    } else {
        ctx.accounts.marketplace.access_mint = None;
    }

    Ok(())
}

