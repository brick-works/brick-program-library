use {
    crate::state::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_spl::{
        token_interface::{Mint, TokenInterface, TokenAccount},
        token::ID as TokenProgramV0,
    }
};

#[derive(Accounts)]
pub struct InitReward<'info> {
    pub system_program: Program<'info, System>,
    #[account(address = TokenProgramV0 @ ErrorCode::IncorrectTokenProgram)]
    pub token_program_v0: Interface<'info, TokenInterface>,
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
        space = REWARD_SIZE,
        seeds = [
            b"reward".as_ref(),
            signer.key().as_ref(),
            marketplace.key().as_ref(),
        ],
        bump,
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
        token::token_program = token_program_v0,
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,
}

pub fn handler<'info>(ctx: Context<InitReward>) -> Result<()> {
    let mut vaults: Vec<Pubkey> = Vec::with_capacity(VAULT_COUNT); 
    vaults.push(ctx.accounts.reward_vault.key());

    let mut vault_bumps: Vec<u8> = Vec::with_capacity(VAULT_COUNT); 
    vault_bumps.push(*ctx.bumps.get("reward_vault").unwrap());

    (*ctx.accounts.reward).authority = ctx.accounts.signer.key();
    (*ctx.accounts.reward).marketplace =  ctx.accounts.marketplace.key();
    (*ctx.accounts.reward).reward_vaults = vaults;
    (*ctx.accounts.reward).bumps = RewardBumps {
        bump: *ctx.bumps.get("reward").unwrap(),
        vault_bumps,
    };
    
    Ok(())
}
