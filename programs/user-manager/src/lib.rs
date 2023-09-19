use {
    anchor_lang::prelude::*,
    anchor_spl::{
        associated_token::AssociatedToken,
        token_interface::{Mint, TokenInterface, TokenAccount},
        token::{transfer, Transfer},
    }
};

declare_id!("8qZHpkF9Ai4BFe7R8g4zXduo6rUmWeJQfEkUaEno57Kf");

#[program]
pub mod user_manager {
    use super::*;

    pub fn init_admin(ctx: Context<InitAdmin>) -> Result<()> {
        (*ctx.accounts.admin).authority = ctx.accounts.signer.key();
        (*ctx.accounts.admin).bump = *ctx.bumps.get("admin").unwrap();

        Ok(())
    }

    pub fn init_user(ctx: Context<InitUser>, id: [u8; 16]) -> Result<()> {
        (*ctx.accounts.user).id = id;
        (*ctx.accounts.user).bump = *ctx.bumps.get("user").unwrap();

        Ok(())
    }

    pub fn init_vault(_ctx: Context<InitVault>) -> Result<()> {
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                Transfer {
                    from: ctx.accounts.user_vault.to_account_info(),
                    to: ctx.accounts.receiver_vault.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            ctx.accounts.user_vault.amount,
        ).map_err(|_| ErrorCode::TransferError)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitAdmin<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = 8 + 32 + 1,
        seeds = [
            b"admin".as_ref(),
        ],
        bump,
    )]
    pub admin: Account<'info, Admin>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(id: [u8; 16])]
pub struct InitUser<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [
            b"admin".as_ref(),
        ],
        bump = admin.bump,
        constraint = admin.authority == signer.key()
            @ ErrorCode::IncorrectAuthority
    )]
    pub admin: Account<'info, Admin>,
    #[account(
        init,
        payer = signer,
        space = USER_SIZE,
        seeds = [
            b"user".as_ref(),
            id.as_ref(),
        ],
        bump,
    )]
    pub user: Account<'info, User>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [
            b"admin".as_ref(),
        ],
        bump = admin.bump,
        constraint = admin.authority == signer.key()
            @ ErrorCode::IncorrectAuthority
    )]
    pub admin: Account<'info, Admin>,
    #[account(
        mut,
        seeds = [
            b"user".as_ref(),
            user.id.as_ref()
        ],
        bump = user.bump,
    )]
    pub user: Account<'info, User>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        seeds = [
            b"vault".as_ref(),
            user.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        token::mint = mint,
        token::authority = user,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        seeds = [
            b"admin".as_ref(),
        ],
        bump = admin.bump,
        constraint = admin.authority == signer.key()
            @ ErrorCode::IncorrectAuthority
    )]
    pub admin: Account<'info, Admin>,
    #[account(
        mut,
        seeds = [
            b"user".as_ref(),
            user.id.as_ref()
        ],
        bump = user.bump,
    )]
    pub user: Account<'info, User>,
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"vault".as_ref(),
            user.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
    )]
    pub user_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub receiver_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[account]
pub struct User {
    pub id: [u8; 16],
    pub bump: u8
}

#[account]
pub struct Admin {
    pub authority: Pubkey,
    pub bump: u8
}

pub const USER_SIZE: usize = 8 + 32 + 32 + 1;

#[error_code]
pub enum ErrorCode {
    #[msg("Wrong associated token account")]
    IncorrectATA,

    #[msg("You are not the authority")]
    IncorrectAuthority,

    #[msg("Incorrect mint")]
    IncorrectMint,

    #[msg("Transfer error")]
    TransferError,

    #[msg("The max size of reward vaults are set at 5")]
    VaultsVectorFull,
}