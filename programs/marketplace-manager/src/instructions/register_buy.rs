use {
    crate::{
        state::*,
        error::ErrorCode,
        utils::pda::*
    },
    anchor_lang::{
        prelude::*,
        system_program::System,
    },    
    anchor_spl::token_interface::{Mint, TokenInterface, TokenAccount},
};

#[derive(Accounts)]
pub struct RegisterBuy<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    /// CHECK: Marketplace is mut because signs sending rewards
    #[account(
        mut,
        address = get_marketplace_address(&marketplace.authority),
        constraint = marketplace.key() == product.marketplace 
            @ ErrorCode::IncorrectAuthority
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        address = get_product_address(&marketplace.key(), &product.id),
        has_one = marketplace
    )]
    pub product: Box<Account<'info, Product>>,
    #[account(
        constraint = payment_mint.key() == product.seller_config.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        constraint = buyer_vault.owner == signer.key()
            @ ErrorCode::IncorrectAuthority,
        constraint = buyer_vault.mint == product.seller_config.payment_mint.key()
            @ ErrorCode::IncorrectATA,
    )]
    pub buyer_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = seller_vault.owner == product.authority 
            @ ErrorCode::IncorrectAuthority,
        constraint = seller_vault.mint == product.seller_config.payment_mint.key()
            @ ErrorCode::IncorrectATA,
    )]
    pub seller_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// ATA that receives fees, should be init before allowing sellers to receive payments with specific mints
    #[account(
        mut,
        constraint = marketplace_vault.owner == marketplace.authority
    )]
    pub marketplace_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    
    /// Note: Reward Mint has to be equal to Payment Mint for decimals and amount consistency
    /// if you are not going to use the reward feat provide bounty_vault as null
    /// this account holds the reward tokens
    #[account(mut)]
    pub bounty_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,

    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler<'info>(ctx: Context<RegisterBuy>, amount: u32) -> Result<()> {
    let total_payment = ctx.accounts.product.seller_config.product_price
        .checked_mul(amount.into()).ok_or(ErrorCode::NumericalOverflow)?;

    if let Some(fees_config) = &ctx.accounts.marketplace.fees_config {
        let marketplace_vault = ctx.accounts.marketplace_vault.as_ref().ok_or(ErrorCode::OptionalAccountNotProvided)?;

        Product::do_fee_payment(
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.buyer_vault.to_account_info(),
            ctx.accounts.seller_vault.to_account_info(),
            marketplace_vault.to_account_info(),
            ctx.accounts.payment_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            fees_config.clone(),
            ctx.accounts.payment_mint.key(),
            total_payment,
            ctx.accounts.payment_mint.decimals,
        )?;
    } else {
        Product::do_payment(
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.buyer_vault.to_account_info(),
            ctx.accounts.seller_vault.to_account_info(),
            ctx.accounts.payment_mint.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            total_payment,
            ctx.accounts.payment_mint.decimals,
        )?;
    }

    if ctx.accounts.marketplace.is_rewards_active(&ctx.accounts.payment_mint.key()) {
        let bounty_vault = ctx.accounts.bounty_vault.as_ref().ok_or(ErrorCode::OptionalAccountNotProvided)?;

        ctx.accounts.marketplace.transfer_bonus(
            bounty_vault.to_account_info(),
            ctx.accounts.payment_mint.to_account_info(),
            ctx.accounts.seller_vault.to_account_info(),
            ctx.accounts.buyer_vault.to_account_info(),
            ctx.accounts.marketplace.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.product.seller_config.product_price,
            ctx.accounts.payment_mint.decimals,
        )?;
    }

    Ok(())
}

