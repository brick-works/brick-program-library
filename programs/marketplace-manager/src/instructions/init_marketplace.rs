use {
    crate::state::*,
    anchor_lang::prelude::*,
    crate::error::ErrorCode,
    crate::utils::mint_builder,
    spl_token_2022::extension::ExtensionType,
    anchor_spl::{
        token_interface::{
            Mint, 
            TokenAccount, 
            TokenInterface
        },
        token_2022::ID as TokenProgram2022,
    },
};

#[derive(Accounts)]
pub struct InitMarketplace<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = Marketplace::SIZE,
        seeds = [Marketplace::get_seeds(&signer.key())],
        bump,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    /// CHECK: this mint is init in the ix handler
    #[account(mut)]  
    pub access_mint: UncheckedAccount<'info>,
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,
    pub discount_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        seeds = [Marketplace::get_vault_seeds(&marketplace.key(), &reward_mint.key())],
        bump,
        token::mint = reward_mint,
        token::authority = marketplace,
        token::token_program = token_program,
    )]
    pub bounty_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    /// Enforcing token22 to make the access token non transferable
    #[account(address = TokenProgram2022 @ ErrorCode::IncorrectTokenProgram)]
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler<'info>(
    ctx: Context<InitMarketplace>, 
    // to use token22 is needed to send the bump from the client (cant init it in the context)
    access_mint_bump: u8,
    fees_config: Option<FeesConfig>,
    rewards_config: Option<RewardsConfig>,
) -> Result<()> {
    if fees_config.is_some() {
        let fees: FeesConfig = fees_config.unwrap();
        Marketplace::validate_fees(&fees)?;
    }

    let authority = ctx.accounts.signer.key();
    let access_mint = Some(ctx.accounts.access_mint.key());
    let bumps = MarketplaceBumps {
        bump: ctx.bumps.marketplace,
        access_mint_bump,
    };

    ctx.accounts.marketplace.initialize(
        authority,
        bumps,
        access_mint,
        fees_config,
        rewards_config,
    )?;

    let extensions: Vec<ExtensionType> = vec![ExtensionType::NonTransferable];
    let signer_mint_authority_seeds: &[&[&[u8]]] = Marketplace::get_signer_seeds(
        &authority, 
        ctx.accounts.marketplace.bumps.bump
    );
    let signer_mint_seeds: &[&[&[u8]]] = Marketplace::get_mint_signer_seeds(
        &ctx.accounts.marketplace.key(), 
        access_mint_bump
    );

    mint_builder(
        signer_mint_seeds,
        signer_mint_authority_seeds,
        extensions,
        ctx.accounts.access_mint.to_account_info(),
        ctx.accounts.marketplace.to_account_info(),
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.rent.clone(),
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    )?;

    Ok(())
}
