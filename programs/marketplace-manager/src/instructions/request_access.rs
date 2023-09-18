use {
    crate::state::*,
    anchor_lang::prelude::*,
};

#[derive(Accounts)]
pub struct RequestAccess<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"marketplace".as_ref(),
            marketplace.authority.as_ref(),
        ],
        bump = marketplace.bumps.bump,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        init,
        payer = signer,
        space = ACCESS_SIZE,
        seeds = [
            b"request".as_ref(),
            signer.key().as_ref(),
            marketplace.key().as_ref(),
        ],
        bump,
    )]
    pub request: Account<'info, Access>,
}

pub fn handler<'info>(ctx: Context<RequestAccess>) -> Result<()> {
    (*ctx.accounts.request).authority = ctx.accounts.signer.key();
    (*ctx.accounts.request).marketplace =  ctx.accounts.marketplace.key();
    (*ctx.accounts.request).bump = *ctx.bumps.get("request").unwrap();
    
    Ok(())
}
