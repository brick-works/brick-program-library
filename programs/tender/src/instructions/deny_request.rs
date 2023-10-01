use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DenyRequest<'info> {
    #[account(mut)]
    pub signer: Signer<'info>
}

pub fn handler<'info>(_ctx: Context<DenyRequest>) -> Result<()> {
    Ok(())
}
