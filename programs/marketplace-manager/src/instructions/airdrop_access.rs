use {
    crate::state::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_spl::{
        token_2022::mint_to,
        token_interface::{Mint, MintTo, TokenInterface, TokenAccount},
        token_2022::ID as TokenProgram2022,
        associated_token::AssociatedToken
    }
};

#[derive(Accounts)]
pub struct AirdropAccess<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub receiver: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"marketplace".as_ref(),
            signer.key().as_ref(),
        ],
        bump = marketplace.bumps.bump,
        constraint = signer.key() == marketplace.authority
            @ErrorCode::IncorrectAuthority,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        mut,
        seeds = [
            b"access_mint".as_ref(),
            marketplace.key().as_ref(),
        ],
        bump = marketplace.bumps.access_mint_bump,
        constraint = access_mint.key() == marketplace.permission_config.access_mint
            @ErrorCode::IncorrectMint
    )]    
    pub access_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = access_mint,
        associated_token::authority = receiver,
        associated_token::token_program = token_program
    )]
    pub access_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    #[account(address = TokenProgram2022 @ ErrorCode::IncorrectTokenProgram)]
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler<'info>(ctx: Context<AirdropAccess>) -> Result<()> {
    let signer_key = ctx.accounts.signer.key();
    let marketplace_seeds = &[
        b"marketplace".as_ref(),
        signer_key.as_ref(),
        &[ctx.accounts.marketplace.bumps.bump],
    ];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.access_mint.to_account_info(),
                to: ctx.accounts.access_vault.to_account_info(),
                authority: ctx.accounts.marketplace.to_account_info(),
            },
            &[&marketplace_seeds[..]],
        ),
        1
    ).map_err(|_| ErrorCode::MintToError)?;
    
    Ok(())
}
