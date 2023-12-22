use {
    crate::utils::pda::*,
    crate::state::*,
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct RequestAccess<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        address = get_marketplace_address(&marketplace.authority)
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        init,
        payer = signer,
        space = Reward::SIZE,
        address = get_access_address(&signer.key(), &marketplace.key())
    )]
    pub request: Account<'info, AccessRequest>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<RequestAccess>) -> Result<()> {
    (*ctx.accounts.request).payer = ctx.accounts.signer.key();
    
    Ok(())
}
