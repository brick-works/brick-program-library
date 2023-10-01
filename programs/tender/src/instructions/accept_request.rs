use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct AcceptRequest<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
}

pub fn handler<'info>(_ctx: Context<AcceptRequest>) -> Result<()> {
    Ok(())
}
