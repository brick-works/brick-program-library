use {
    crate::state::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token_interface::{Mint, TokenInterface, TokenAccount},
        token::ID as TokenProgramV0,
    }
};

#[derive(Accounts)]
pub struct InitBounty<'info> {
    pub system_program: Program<'info, System>,
    #[account(address = TokenProgramV0 @ ErrorCode::IncorrectTokenProgram)]
    pub token_program_v0: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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
        constraint = signer.key() == marketplace.authority
            @ErrorCode::IncorrectAuthority
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        seeds = [
            b"bounty_vault".as_ref(), 
            marketplace.key().as_ref(),
            reward_mint.key().as_ref(),
        ],
        bump,
        token::mint = reward_mint,
        token::authority = marketplace,
        token::token_program = token_program_v0,
    )]
    pub bounty_vault: Box<InterfaceAccount<'info, TokenAccount>>,
}

pub fn handler<'info>(ctx: Context<InitBounty>,) -> Result<()> {
    if ctx.accounts.marketplace.rewards_config.bounty_vaults.len() >= VAULT_COUNT {
        return Err(ErrorCode::VaultsVectorFull.into());
    }

    ctx.accounts.marketplace.rewards_config.bounty_vaults.push(ctx.accounts.bounty_vault.key());
    ctx.accounts.marketplace.bumps.vault_bumps.push(*ctx.bumps.get("bounty_vault").unwrap());
    
    Ok(())
}
