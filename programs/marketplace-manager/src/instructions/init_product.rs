use {
    crate::state::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_lang::system_program::System,
    anchor_spl::token_interface::{
        Mint,
        TokenAccount
    },
};

#[derive(Accounts)]
pub struct InitProduct<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [Marketplace::get_seeds(&signer.key())],
        bump = marketplace.bumps.bump,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        init,
        payer = signer,
        space = Product::SIZE,
        seeds = [Product::get_seeds(&signer.key())],
        bump,
    )]
    pub product: Box<Account<'info, Product>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [Marketplace::get_mint_seeds(&marketplace.key())],
        bump = marketplace.bumps.access_mint_bump,
    )]    
    pub access_mint: Box<InterfaceAccount<'info, Mint>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    /// CHECK: validated on the ix logic
    /// needs to be optional, permisionless marketplaces have to provide a null address
    #[account(mut)]    
    pub access_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
}

pub fn handler<'info>(
    ctx: Context<InitProduct>,     
    id: [u8; 16],
    product_price: u64
) -> Result<()> {
    if ctx.accounts.marketplace.access_mint.is_some() {
        let access_vault = ctx.accounts.access_vault.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;
        let on_chain_mint = &ctx.accounts.marketplace.access_mint.unwrap();

        if !Marketplace::validate_access(
            &on_chain_mint,
            &ctx.accounts.access_mint.key(),
            &access_vault.owner,
            &ctx.accounts.signer.key(),
            access_vault.amount,
        ) {
            return Err(ErrorCode::NotAllowed.into());
        }
    }

    let authority = ctx.accounts.product.key();
    let payment_mint = ctx.accounts.payment_mint.key();
    ctx.accounts.product.initialize(
        authority,
        id,
        SellerConfig {
            payment_mint,
            product_price,
        },
        ctx.bumps.product,
    )?;

    Ok(())
}
