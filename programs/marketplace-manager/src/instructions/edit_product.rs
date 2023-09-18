use {
    crate::state::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_spl::token_interface::Mint
};

#[derive(Accounts)]
pub struct EditProduct<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"product".as_ref(),
            product.first_id.as_ref(),
            product.second_id.as_ref(),
            product.marketplace.as_ref(),
        ],
        bump = product.bumps.bump,
        constraint = signer.key() == product.authority 
            @ ErrorCode::IncorrectAuthority,
    )]
    pub product: Box<Account<'info, Product>>,
    /// CHECK: no need to validate, seller is the unique wallet who can call this instruction
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
}

pub fn handler<'info>(ctx: Context<EditProduct>, product_price: u64) -> Result<()> {
    (*ctx.accounts.product).seller_config = SellerConfig {
        payment_mint: ctx.accounts.payment_mint.key(),
        product_price,
    };
    
    Ok(())
}