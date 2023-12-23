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
        address = get_marketplace_address(&signer.key()),
        constraint = marketplace_authority.key() == marketplace.authority 
            @ ErrorCode::IncorrectAuthority
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(address = get_product_address(&product.id))]
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
    /// ATA that receives fees
    /// in case it does not exist, the user will pay for that
    /// discourages the use of unusual tokens
    #[account(
        init_if_needed,
        payer = signer,
        token::mint = payment_mint,
        token::authority = marketplace_authority,
        token::token_program = token_program,
    )]
    pub marketplace_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// included to create ata if needed (validated in the marketplace account)
    #[account(mut)]
    pub marketplace_authority: SystemAccount<'info>,
    
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
        Product::do_fee_payment(
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.buyer_vault.to_account_info(),
            ctx.accounts.seller_vault.to_account_info(),
            ctx.accounts.marketplace_vault.to_account_info(),
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

