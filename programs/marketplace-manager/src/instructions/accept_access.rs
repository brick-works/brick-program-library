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

/// CHECK: this instruction only needs the marketplace authority == signer validation
#[derive(Accounts)]
pub struct AcceptAccess<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    // Needed to send back the rent fee of the request account
    #[account(mut)]
    pub requestor: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [Marketplace::get_seeds(&signer.key())],
        bump = marketplace.bumps.bump,
        constraint = signer.key() == marketplace.authority
            @ErrorCode::IncorrectAuthority,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        mut,
        seeds = [AccessRequest::get_seeds(&requestor.key(), &marketplace.key())],
        bump = access_request.bump,
        close = requestor,
    )]
    pub access_request: Account<'info, AccessRequest>,
    #[account(
        mut,
        seeds = [Marketplace::get_mint_seeds(&marketplace.key())],
        bump = marketplace.bumps.access_mint_bump,
    )]
    pub access_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = access_mint,
        associated_token::authority = requestor,
        associated_token::token_program = token_program
    )]
    pub access_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    /// Enforcing token22 to make the access token non transferable
    #[account(address = TokenProgram2022 @ ErrorCode::IncorrectTokenProgram)]
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler<'info>(ctx: Context<AcceptAccess>) -> Result<()> {
    let signer_seeds = Marketplace::get_signer_seeds(
        &ctx.accounts.marketplace.key(), 
        ctx.accounts.marketplace.bumps.bump
    );

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.access_mint.to_account_info(),
                to: ctx.accounts.access_vault.to_account_info(),
                authority: ctx.accounts.marketplace.to_account_info(),
            },
            signer_seeds,
        ),
        1
    ).map_err(|_| ErrorCode::MintToError)?;
    
    Ok(())
}
