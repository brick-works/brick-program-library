use {
    crate::state::{Marketplace, AccessRequest},
    crate::error::ErrorCode,
    crate::utils::pda::*,
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
        address = get_marketplace_address(&signer.key()),
        constraint = signer.key() == marketplace.authority
            @ErrorCode::IncorrectAuthority,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        mut,
        address = get_access_address(&requestor.key(), &marketplace.key()),
        close = requestor,
    )]
    pub access_request: Account<'info, AccessRequest>,
    #[account(
        mut,
        address = get_access_mint_address(&marketplace.key()),
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
    let marketplace_seeds = &[
        "marketplace".as_ref(),
        ctx.accounts.marketplace.authority.as_ref(),
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
    )?;
    
    Ok(())
}
