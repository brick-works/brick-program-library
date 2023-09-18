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
pub struct InitRewardVault<'info> {
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
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        mut,
        seeds = [
            b"reward".as_ref(),
            signer.key().as_ref(),
            marketplace.key().as_ref(),
        ],
        bump = reward.bumps.bump,
    )]
    pub reward: Account<'info, Reward>,
    #[account(
        constraint = reward_mint.key() == marketplace.rewards_config.reward_mint 
            @ ErrorCode::IncorrectMint
    )]
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
        token::token_program = token_program_v0,
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,
}

pub fn handler<'info>(ctx: Context<InitRewardVault>,) -> Result<()> {
    if ctx.accounts.reward.reward_vaults.len() >= VAULT_COUNT {
        return Err(ErrorCode::VaultsVectorFull.into());
    }

    ctx.accounts.reward.reward_vaults.push(ctx.accounts.reward_vault.key());
    ctx.accounts.reward.bumps.vault_bumps.push(*ctx.bumps.get("reward_vault").unwrap());
    
    Ok(())
}

