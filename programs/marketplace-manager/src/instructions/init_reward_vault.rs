use {
    crate::state::*,
    crate::utils::pda::*,
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token_interface::{Mint, TokenInterface, TokenAccount},
    }
};

#[derive(Accounts)]
pub struct InitRewardVault<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        address = get_marketplace_address(&marketplace.authority),
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        mut,
        address = get_reward_address(&signer.key(), &marketplace.key()),
    )]
    pub reward: Account<'info, Reward>,
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        seeds = [
            b"reward_vault".as_ref(),
            signer.key().as_ref(),
            marketplace.key().as_ref(),
            reward_mint.key().as_ref(),
        ],
        bump,
        token::mint = reward_mint,
        token::authority = reward,
        token::token_program = token_program,
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler<'info>(_ctx: Context<InitRewardVault>,) -> Result<()> {
    Ok(())
}

