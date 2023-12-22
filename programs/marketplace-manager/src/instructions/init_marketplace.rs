use {
    crate::state::*,
    anchor_lang::prelude::*,
    crate::utils::pda::*,
    crate::error::ErrorCode,
    crate::utils::mint_builder,
    spl_token_2022::extension::ExtensionType,
    anchor_spl::{
        token_interface::TokenInterface,
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
        address = get_marketplace_address(&signer.key()),
        // included to be the bump available in the context
        // needed to be on-chain to sign with the pda
        seeds = [b"marketplace".as_ref(), signer.key().as_ref()],
        bump
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    /// CHECK: this mint is init in the ix handler
    #[account(
        mut,
        address = get_access_address(&signer.key(), &marketplace.key()),
    )]  
    pub access_mint: UncheckedAccount<'info>,
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
    let authority = ctx.accounts.signer.key();
    let marketplace_key = ctx.accounts.marketplace.key();
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

    if ctx.accounts.marketplace.fees_config.is_some() {
        ctx.accounts.marketplace.validate_fees()?;
    }
    if ctx.accounts.marketplace.rewards_config.is_some() {
        ctx.accounts.marketplace.validate_rewards()?;
    }

    let extensions: Vec<ExtensionType> = vec![ExtensionType::NonTransferable];
    let mint_seeds: &[&[u8]] = &[
        b"access_mint",
        marketplace_key.as_ref(),
        &[access_mint_bump]
    ];
    let marketplace_seeds = &[
        b"marketplace".as_ref(),
        ctx.accounts.marketplace.authority.as_ref(),
        &[ctx.accounts.marketplace.bumps.bump],
    ];

    mint_builder(
        mint_seeds,
        marketplace_seeds,
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
