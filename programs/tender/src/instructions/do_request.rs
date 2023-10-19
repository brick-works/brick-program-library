use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::token_interface::Mint,
};

#[derive(Accounts)]
pub struct DoRequest<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"proposal".as_ref(),
            proposal.id.as_ref(),
        ],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        init,
        space = REQUEST_SIZE,
        payer = signer,
        seeds = [
            b"request".as_ref(),
            proposal.key().as_ref(),
            signer.key().as_ref(),
        ],
        bump
    )]
    pub request: Account<'info, Request>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<DoRequest>, price: u64) -> Result<()> {
    (*ctx.accounts.request).price = price;
    (*ctx.accounts.request).bump = ctx.bumps.request;

    Ok(())
}
