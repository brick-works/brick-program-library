use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DoRequest<'info> {
    #[account(mut)]
    pub signer: Signer<'info>
}

pub fn handler<'info>(_ctx: Context<DoRequest>) -> Result<()> {
    Ok(())
}
