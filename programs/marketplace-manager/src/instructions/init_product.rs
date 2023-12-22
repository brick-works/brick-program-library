use {
    crate::state::*,
    crate::error::ErrorCode,
    crate::utils::pda::*,
    anchor_lang::prelude::*,
    anchor_lang::system_program::System,
    anchor_spl::token_interface::{
        Mint,
        TokenAccount
    },
};

#[derive(Accounts)]
#[instruction(id: [u8; 16])]
pub struct InitProduct<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        address = get_marketplace_address(&signer.key()),
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        init,
        payer = signer,
        space = Product::SIZE,
        address = get_product_address(&product.id),
    )]
    pub product: Box<Account<'info, Product>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        address = get_access_mint_address(&marketplace.key()),
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
    )?;

    Ok(())
}
