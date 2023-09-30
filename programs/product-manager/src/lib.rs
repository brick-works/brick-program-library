use {
    anchor_lang::prelude::*,
    anchor_spl::{
        token_interface::{ Mint, TokenInterface, TokenAccount },
        associated_token::AssociatedToken,
        token::{ transfer, Transfer },
        token_interface::{ CloseAccount, close_account },
    }
};

declare_id!("ESb8CKVxVNpDS3c1fsrWwmMkfKga7Z9pdAdbKU5Lv3VU");

#[program]
pub mod product_manager {
    use super::*;

    pub fn init_product(ctx: Context<InitProduct>, id: [u8; 16], price: u64) -> Result<()> {
        (*ctx.accounts.product).id = id;
        (*ctx.accounts.product).authority = ctx.accounts.signer.key();
        (*ctx.accounts.product).payment_mint = ctx.accounts.payment_mint.key();
        (*ctx.accounts.product).price = price;
        (*ctx.accounts.product).bump = *ctx.bumps.get("product").unwrap();

        emit!(ProductEvent {
            address: ctx.accounts.product.key().to_string(),
            mint: ctx.accounts.payment_mint.key().to_string(),
            seller: ctx.accounts.signer.key().to_string(),
            price: ctx.accounts.product.price,
            blocktime: Clock::get().unwrap().unix_timestamp
        });

        Ok(())
    }

    pub fn escrow_pay(ctx: Context<EscrowPay>, product_amount: u64, expire_time: i64) -> Result<()> {
        (*ctx.accounts.escrow).payer = ctx.accounts.signer.key();
        (*ctx.accounts.escrow).receiver = ctx.accounts.seller.key();
        (*ctx.accounts.escrow).product = ctx.accounts.product.key();
        (*ctx.accounts.escrow).product_amount = product_amount;
        let now = Clock::get().unwrap().unix_timestamp;
        (*ctx.accounts.escrow).expire_time = now + expire_time;
        (*ctx.accounts.escrow).vault_bump = *ctx.bumps.get("escrow_vault").unwrap();
        (*ctx.accounts.escrow).bump = *ctx.bumps.get("escrow").unwrap();

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                Transfer {
                    from: ctx.accounts.transfer_vault.to_account_info(),
                    to: ctx.accounts.escrow_vault.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            ctx.accounts.product.price * product_amount,
        )?;

        emit!(EscrowEvent {
            address: ctx.accounts.escrow.key().to_string(),
            vault: ctx.accounts.escrow_vault.key().to_string(),
            mint: ctx.accounts.payment_mint.key().to_string(),
            payer: ctx.accounts.signer.key().to_string(),
            receiver: ctx.accounts.seller.key().to_string(),
            product: ctx.accounts.product.key().to_string(),
            amount: ctx.accounts.product.price,
            product_amount: ctx.accounts.escrow.product_amount,
            expire_time: ctx.accounts.escrow.expire_time,
            blocktime: Clock::get().unwrap().unix_timestamp
        });

        Ok(())
    }

    pub fn direct_pay(ctx: Context<DirectPay>, product_amount: u64) -> Result<()> {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            ctx.accounts.product.price * product_amount,
        )?;

        emit!(DirectPayEvent {
            mint: ctx.accounts.payment_mint.key().to_string(),
            payer: ctx.accounts.signer.key().to_string(),
            receiver: ctx.accounts.seller.key().to_string(),
            product: ctx.accounts.product.key().to_string(),
            amount: ctx.accounts.product.price,
            product_amount: product_amount,
            blocktime: Clock::get().unwrap().unix_timestamp
        });

        Ok(())
    }

    pub fn accept(ctx: Context<Accept>) -> Result<()> {
        let now = Clock::get().unwrap().unix_timestamp;
        if now > ctx.accounts.escrow.expire_time {
            return Err(ErrorCode::TimeExpired.into());
        }

        let product_key = ctx.accounts.product.key();
        let buyer_key = ctx.accounts.buyer.key();
    
        let escrow_seeds = [
            b"escrow".as_ref(),
            product_key.as_ref(),
            buyer_key.as_ref(),
            &[ctx.accounts.escrow.bump],
        ];
    
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                Transfer {
                    from: ctx.accounts.escrow_vault.to_account_info(),
                    to: ctx.accounts.transfer_vault.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                &[&escrow_seeds[..]]
            ),
            ctx.accounts.escrow_vault.amount
        )?;

        emit!(SellerResponseEvent {
            response: SellerResponse::Accept,
            escrow: ctx.accounts.escrow.key().to_string(),
            mint: ctx.accounts.payment_mint.key().to_string(),
            payer: ctx.accounts.buyer.key().to_string(),
            receiver: ctx.accounts.signer.key().to_string(),
            product: ctx.accounts.product.key().to_string(),
            amount: ctx.accounts.escrow_vault.amount,
            product_amount: ctx.accounts.escrow.product_amount,
            blocktime: now
        });

        close_account( 
            CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            CloseAccount {
                account: ctx.accounts.escrow_vault.to_account_info(),
                destination: ctx.accounts.buyer.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[&escrow_seeds[..]],
        ))?;

        Ok(())
    }

    pub fn deny(ctx: Context<Deny>) -> Result<()> {
        let now = Clock::get().unwrap().unix_timestamp;
        if now > ctx.accounts.escrow.expire_time {
            return Err(ErrorCode::TimeExpired.into());
        }

        let product_key = ctx.accounts.product.key();
        let buyer_key = ctx.accounts.buyer.key();
    
        let escrow_seeds = [
            b"escrow".as_ref(),
            product_key.as_ref(),
            buyer_key.as_ref(),
            &[ctx.accounts.escrow.bump],
        ];
    
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                Transfer {
                    from: ctx.accounts.escrow_vault.to_account_info(),
                    to: ctx.accounts.transfer_vault.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                &[&escrow_seeds[..]]
            ),
            ctx.accounts.escrow_vault.amount
        )?;

        close_account( 
            CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            CloseAccount {
                account: ctx.accounts.escrow_vault.to_account_info(),
                destination: ctx.accounts.buyer.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[&escrow_seeds[..]],
        ))?;

        emit!(SellerResponseEvent {
            response: SellerResponse::Deny,
            escrow: ctx.accounts.escrow.key().to_string(),
            mint: ctx.accounts.payment_mint.key().to_string(),
            payer: ctx.accounts.buyer.key().to_string(),
            receiver: ctx.accounts.buyer.key().to_string(),
            product: ctx.accounts.product.key().to_string(),
            amount: ctx.accounts.escrow_vault.amount,
            product_amount: ctx.accounts.escrow.product_amount,
            blocktime: now
        });

        Ok(())
    }

    pub fn recover_funds(ctx: Context<RecoverFunds>) -> Result<()> {
        let now = Clock::get().unwrap().unix_timestamp;
        if now < ctx.accounts.escrow.expire_time {
            return Err(ErrorCode::CannotRecoverYet.into());
        }

        let product_key = ctx.accounts.product.key();
        let signer_key = ctx.accounts.signer.key();
    
        let escrow_seeds = [
            b"escrow".as_ref(),
            product_key.as_ref(),
            signer_key.as_ref(),
            &[ctx.accounts.escrow.bump],
        ];
    
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                Transfer {
                    from: ctx.accounts.escrow_vault.to_account_info(),
                    to: ctx.accounts.transfer_vault.to_account_info(),
                    authority: ctx.accounts.escrow.to_account_info(),
                },
                &[&escrow_seeds[..]]
            ),
            ctx.accounts.escrow_vault.amount
        )?;

        close_account( 
            CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(), 
            CloseAccount {
                account: ctx.accounts.escrow_vault.to_account_info(),
                destination: ctx.accounts.signer.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            },
            &[&escrow_seeds[..]],
        ))?;

        emit!(RecoverEvent {
            escrow: ctx.accounts.escrow.key().to_string(),
            seller: ctx.accounts.seller.key().to_string(),
            buyer: ctx.accounts.signer.key().to_string(),
            amount: ctx.accounts.escrow_vault.amount
        });

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(id: [u8; 16])]
pub struct InitProduct<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        space = PRODUCT_SIZE,
        payer = signer,
        seeds = [
            b"product".as_ref(),
            signer.key().as_ref(),
            id.as_ref()
        ],
        bump
    )]
    pub product: Account<'info, Product>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DirectPay<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub seller: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"product".as_ref(),
            seller.key().as_ref(),
            product.id.as_ref()
        ],
        bump = product.bump,
        constraint = product.authority == seller.key()
            @ ErrorCode::IncorrectAuthority,
        constraint = product.payment_mint == payment_mint.key()
            @ ErrorCode::IncorrectMint
    )]
    pub product: Account<'info, Product>,
    #[account(
        mut,
        constraint = from.owner == signer.key()
            @ ErrorCode::IncorrectOwner,
        constraint = from.mint == product.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub from: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = to.owner == seller.key()
            @ ErrorCode::IncorrectOwner,
        constraint = to.mint == product.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub to: Box<InterfaceAccount<'info, TokenAccount>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct EscrowPay<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub seller: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"product".as_ref(),
            seller.key().as_ref(),
            product.id.as_ref()
        ],
        bump = product.bump,
        constraint = product.authority == seller.key()
            @ ErrorCode::IncorrectAuthority,
        constraint = product.payment_mint == payment_mint.key()
            @ ErrorCode::IncorrectMint
    )]
    pub product: Account<'info, Product>,
    #[account(
        init,
        payer = signer,
        space = ESCORW_SIZE,
        seeds = [
            b"escrow".as_ref(),
            product.key().as_ref(),
            signer.key().as_ref()
        ],
        bump
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        init,
        payer = signer,
        seeds = [
            b"escrow_vault".as_ref(),
            product.key().as_ref(),
            signer.key().as_ref()
        ],
        bump,
        token::mint = payment_mint,
        token::authority = escrow,
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = transfer_vault.owner == signer.key()
            @ ErrorCode::IncorrectOwner,
        constraint = transfer_vault.mint == product.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub transfer_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Accept<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub buyer: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"escrow".as_ref(),
            product.key().as_ref(),
            buyer.key().as_ref()
        ],
        bump = escrow.bump,
        constraint = escrow.receiver == signer.key()
            @ ErrorCode::IncorrectAuthority,
        constraint = escrow.product == product.key()
            @ ErrorCode::IncorrectProduct,
        close = buyer,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        seeds = [
            b"product".as_ref(),
            signer.key().as_ref(),
            product.id.as_ref()        
        ],
        bump = product.bump,
        constraint = signer.key() == product.authority 
            @ ErrorCode::IncorrectAuthority,
        constraint = product.payment_mint == payment_mint.key()
            @ ErrorCode::IncorrectMint
    )]
    pub product: Account<'info, Product>,
    #[account(
        mut,
        seeds = [
            b"escrow_vault".as_ref(),
            product.key().as_ref(),
            buyer.key().as_ref()
        ],
        bump = escrow.vault_bump,
        constraint = escrow_vault.owner == escrow.key()
            @ ErrorCode::IncorrectOwner,
        constraint = escrow_vault.mint == payment_mint.key()
            @ ErrorCode::IncorrectMint
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = transfer_vault.owner == signer.key()
            @ ErrorCode::IncorrectOwner,
        constraint = transfer_vault.mint == product.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub transfer_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Deny<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub buyer: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"escrow".as_ref(),
            product.key().as_ref(),
            buyer.key().as_ref()
        ],
        bump = escrow.bump,
        constraint = escrow.payer == buyer.key()
            @ ErrorCode::IncorrectAuthority,
        constraint = escrow.product == product.key()
            @ ErrorCode::IncorrectProduct,
        close = buyer,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        seeds = [
            b"product".as_ref(),
            signer.key().as_ref(),
            product.id.as_ref()        
        ],
        bump = product.bump,
        constraint = signer.key() == product.authority 
            @ ErrorCode::IncorrectAuthority,
        constraint = product.payment_mint == payment_mint.key()
            @ ErrorCode::IncorrectMint
    )]
    pub product: Account<'info, Product>,
    #[account(
        mut,
        seeds = [
            b"escrow_vault".as_ref(),
            product.key().as_ref(),
            buyer.key().as_ref()
        ],
        bump = escrow.vault_bump,
        constraint = escrow_vault.owner == escrow.key()
            @ ErrorCode::IncorrectOwner,
        constraint = escrow_vault.mint == payment_mint.key()
            @ ErrorCode::IncorrectMint
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = transfer_vault.owner == buyer.key()
            @ ErrorCode::IncorrectOwner,
        constraint = transfer_vault.mint == product.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub transfer_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct RecoverFunds<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    pub seller: SystemAccount<'info>,    
    #[account(
        mut,
        seeds = [
            b"escrow".as_ref(),
            product.key().as_ref(),
            signer.key().as_ref()
        ],
        bump = escrow.bump,
        constraint = escrow.payer == signer.key() 
            @ ErrorCode::IncorrectAuthority,
        constraint = escrow.product == product.key()
            @ ErrorCode::IncorrectProduct,
        close = signer,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        seeds = [
            b"product".as_ref(),
            seller.key().as_ref(),
            product.id.as_ref()        
        ],
        bump = product.bump,
        constraint = seller.key() == product.authority 
            @ ErrorCode::IncorrectAuthority,
        constraint = product.payment_mint == payment_mint.key()
            @ ErrorCode::IncorrectMint
    )]
    pub product: Account<'info, Product>,
    #[account(
        mut,
        seeds = [
            b"escrow_vault".as_ref(),
            product.key().as_ref(),
            signer.key().as_ref()
        ],
        bump = escrow.vault_bump,
        constraint = escrow_vault.owner == escrow.key()
            @ ErrorCode::IncorrectOwner,
        constraint = escrow_vault.mint == product.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = transfer_vault.owner == signer.key()
            @ ErrorCode::IncorrectOwner,
        constraint = transfer_vault.mint == product.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub transfer_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[account]
pub struct Product {
    pub id: [u8; 16],
    pub authority: Pubkey,
    pub payment_mint: Pubkey,
    pub price: u64,
    pub bump: u8
}

pub const PRODUCT_SIZE: usize = 8 + 16 + 32 + 32 + 8 + 1;

#[account]
pub struct Escrow {
    /// depending on the blocktime the authority is the buyer or the seller
    /// seller can accept or deny propossal before expire time
    /// buyer can recover funds after expire time
    pub payer: Pubkey,
    pub receiver: Pubkey,
    pub product: Pubkey,
    pub product_amount: u64,
    pub expire_time: i64,
    pub vault_bump: u8,
    pub bump: u8,
}

pub const ESCORW_SIZE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 1 + 1;

#[event]
pub struct ProductEvent {
    pub address: String,
    pub mint: String,
    pub seller: String,
    pub price: u64,
    pub blocktime: i64,
} 

#[event]
pub struct EscrowEvent {
    pub address: String,
    pub vault: String,
    pub mint: String,
    pub payer: String,
    pub receiver: String,
    pub product: String,
    pub amount: u64,
    pub product_amount: u64,
    pub expire_time: i64,
    pub blocktime: i64,
}

#[event]
pub struct DirectPayEvent {
    pub mint: String,
    pub payer: String,
    pub receiver: String,
    pub product: String,
    pub amount: u64,
    pub product_amount: u64,
    pub blocktime: i64,
}

#[event]
pub struct SellerResponseEvent{
    pub response: SellerResponse,
    pub escrow: String,
    pub mint: String,
    pub payer: String,
    pub receiver: String,
    pub product: String,
    pub amount: u64,
    pub product_amount: u64,
    pub blocktime: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum SellerResponse {
    Accept,
    Deny,
}

#[event]
pub struct RecoverEvent {
    pub escrow: String,
    pub seller: String,
    pub buyer: String,
    pub amount: u64
}

#[error_code]
pub enum ErrorCode {
    #[msg("Wrong authority")]
    IncorrectAuthority,
    #[msg("Wrong owner on a token account")]
    IncorrectOwner,
    #[msg("Wrong mint on a token account")]
    IncorrectMint,
    #[msg("Wrong product on a escrow")]
    IncorrectProduct,
    #[msg("Your time to accept or deny propossal has expired")]
    TimeExpired,
    #[msg("Payment recovery is not allowed at this time")]
    CannotRecoverYet,
}