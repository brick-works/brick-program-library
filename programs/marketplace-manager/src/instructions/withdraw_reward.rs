use {
    crate::state::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_spl::token::{transfer, Transfer},
    anchor_spl::{
        token_interface::{Mint, TokenInterface, TokenAccount},
        token::ID,
    }
};

#[derive(Accounts)]
pub struct WithdrawReward<'info> {
    #[account(address = ID @ ErrorCode::IncorrectTokenProgram)]
    pub token_program_v0: Interface<'info, TokenInterface>,    
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"marketplace".as_ref(),
            marketplace.authority.as_ref(),
        ],
        bump = marketplace.bumps.bump,
        //has_one = governance_mint @ ErrorCode::IncorrectMint,
        //has_one = governance_bonus_vault @ ErrorCode::IncorrectATA,
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
        constraint = signer.key() == reward.authority @ ErrorCode::IncorrectAuthority
    )]
    pub reward: Account<'info, Reward>,
    /// CHECK: validated in the governance account contraints
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        constraint = receiver_vault.owner == reward.authority @ ErrorCode::IncorrectAuthority,
        //constraint = receiver_vault.mint == governance_mint.key() @ ErrorCode::IncorrectMint,    
    )]
    pub receiver_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [
            b"reward_vault".as_ref(),
            signer.key().as_ref(),
            marketplace.key().as_ref(),
            reward_mint.key().as_ref()
        ],
        bump = get_bump(
            reward_vault.key(), 
            reward.bumps.clone(), 
            reward.reward_vaults.clone()
        ),
        constraint = reward_vault.owner == reward.key() 
            @ ErrorCode::IncorrectAuthority,
        constraint = reward_vault.mint == reward_mint.key() 
            @ ErrorCode::IncorrectMint,
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,
}

pub fn handler<'info>(ctx: Context<WithdrawReward>) -> Result<()> {
    if ctx.accounts.marketplace.rewards_config.rewards_enabled {
        return Err(ErrorCode::OpenPromotion.into());
    } 

    let signer_key = ctx.accounts.signer.key().to_bytes();
    let marketplace_key = ctx.accounts.marketplace.key().to_bytes();
    let bump_array = [ctx.accounts.reward.bumps.bump];
    let seeds = &[
        b"reward".as_ref(),
        &signer_key,
        &marketplace_key,
        &bump_array,
    ];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program_v0.to_account_info(),
            Transfer {
                from: ctx.accounts.reward_vault.to_account_info(),
                to: ctx.accounts.receiver_vault.to_account_info(),
                authority: ctx.accounts.reward.to_account_info(),
            },
            &[&seeds[..]],
        ),
        ctx.accounts.reward_vault.amount,
    ).map_err(|_| ErrorCode::TransferError)?;
    
    Ok(())
}

pub fn get_bump(address: Pubkey, reward_bumps: RewardBumps, reward_vaults: Vec<Pubkey>) -> u8 {
    reward_vaults.iter().position(|&r| r == address)
        .map(|index| reward_bumps.vault_bumps[index])
        .unwrap_or(0)
}