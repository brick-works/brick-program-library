use {
    crate::state::*,
    crate::utils::pda::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token_interface::{Mint, TokenInterface, TokenAccount},
    }
};

#[derive(Accounts)]
pub struct InitBounty<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        address = get_marketplace_address(&signer.key()),
        constraint = signer.key() == marketplace.authority
            @ErrorCode::IncorrectAuthority
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        address = get_bounty_address(&marketplace.key(), &reward_mint.key()),
        token::mint = reward_mint,
        token::authority = marketplace,
    )]
    pub bounty_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler<'info>(_ctx: Context<InitBounty>,) -> Result<()> {
    Ok(())
}
