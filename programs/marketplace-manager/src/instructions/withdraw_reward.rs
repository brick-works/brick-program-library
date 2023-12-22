use {
    crate::state::*,
    crate::error::ErrorCode,
    crate::utils::pda::*,
    anchor_lang::prelude::*,
    anchor_spl::token::{transfer, Transfer},
    anchor_spl::token_interface::{Mint, TokenInterface, TokenAccount}
};

#[derive(Accounts)]
pub struct WithdrawReward<'info> { 
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(address = get_marketplace_address(&signer.key()))]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        mut,
        address = get_reward_address(&signer.key(), &marketplace.key()),
        constraint = signer.key() == reward.authority @ ErrorCode::IncorrectAuthority
    )]
    pub reward: Account<'info, Reward>,
    /// CHECK: validated in the governance account contraints
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        constraint = receiver_vault.owner == reward.authority @ ErrorCode::IncorrectAuthority,
    )]
    pub receiver_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = reward_vault.owner == reward.key() 
            @ ErrorCode::IncorrectAuthority,
        constraint = reward_vault.mint == reward_mint.key() 
            @ ErrorCode::IncorrectMint,
    )]
    pub reward_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,   
}

pub fn handler<'info>(ctx: Context<WithdrawReward>) -> Result<()> {
    let signer_key = ctx.accounts.signer.key().to_bytes();
    let marketplace_key = ctx.accounts.marketplace.key().to_bytes();
    let seeds = &[
        b"reward".as_ref(),
        &signer_key,
        &marketplace_key,
        &[ctx.accounts.reward.bump],
    ];

    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
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