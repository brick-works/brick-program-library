use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{
        token_interface::{ Mint, TokenAccount, TokenInterface },
        token::{ transfer, Transfer, mint_to, MintTo },
        associated_token::AssociatedToken
    }
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"network".as_ref()],
        bump = network.bump
    )]
    pub network: Account<'info, Network>,
    #[account(
        mut,
        seeds = [
            b"proposal".as_ref(),
            proposal.id.as_ref()
        ],
        bump = proposal.bump
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        seeds = [
            b"vault".as_ref(),
            proposal.key().as_ref(),
        ],
        bump = proposal.vault_bump
    )]
    pub proposal_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = network_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub receiver_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"network_mint".as_ref(),
            network.key().as_ref(),
        ],
        bump = network.mint_bump,
    )]
    pub network_mint: Box<InterfaceAccount<'info, Mint>>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler<'info>(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            Transfer {
                from: ctx.accounts.deposit_vault.to_account_info(),
                to: ctx.accounts.proposal_vault.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        amount,
    )?;

    let network_seeds = &[
        b"network".as_ref(),
        &[ctx.accounts.network.bump],
    ];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.network_mint.to_account_info(),
                to: ctx.accounts.receiver_vault.to_account_info(),
                authority: ctx.accounts.network.to_account_info(),
            },
            &[&network_seeds[..]],
        ),
        amount
    )?;

    Ok(())
}
