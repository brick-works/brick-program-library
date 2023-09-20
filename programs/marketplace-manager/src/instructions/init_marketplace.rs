use {
    crate::state::*,
    crate::utils::assert_derivation,
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
        token::ID as TokenProgramV0,
        token_2022::ID as TokenProgram2022,
    },
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitMarketplaceParams {
    pub fee: u16,
    pub fee_reduction: u16,
    pub seller_reward: u16,
    pub buyer_reward: u16,
    pub transferable: bool,
    pub permissionless: bool,
    pub rewards_enabled: bool,
    pub access_mint_bump: u8,
    pub fee_payer: PaymentFeePayer,
}

#[derive(Accounts)]
#[instruction(params: InitMarketplaceParams)]
pub struct InitMarketplace<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = MARKETPLACE_SIZE,
        seeds = [
            b"marketplace".as_ref(),
            signer.key().as_ref(),
        ],
        bump,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    /// CHECK: this mint is init in the instruction logic
    #[account(
        mut,
        seeds = [
            b"access_mint".as_ref(),
            marketplace.key().as_ref(),
        ],
        bump = params.access_mint_bump,
    )]  
    pub access_mint: AccountInfo<'info>,
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,
    pub discount_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        seeds = [
            b"bounty_vault".as_ref(), 
            marketplace.key().as_ref(),
            reward_mint.key().as_ref(),
        ],
        bump,
        token::mint = reward_mint,
        token::authority = marketplace,
        token::token_program = token_program,
    )]
    pub bounty_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    #[account(address = TokenProgram2022 @ ErrorCode::IncorrectTokenProgram)]
    pub token_program_2022: Interface<'info, TokenInterface>,
    #[account(address = TokenProgramV0 @ ErrorCode::IncorrectTokenProgram)]
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler<'info>(ctx: Context<InitMarketplace>, params: InitMarketplaceParams) -> Result<()> {
    if params.fee_reduction > 10000 || params.fee > 10000 || params.seller_reward > 10000 || params.buyer_reward > 10000 {
        return Err(ErrorCode::IncorrectFee.into());
    }

    let signer_key = ctx.accounts.signer.key();
    let marketplace_key = ctx.accounts.marketplace.key();
    let mint_seeds: &[&[u8]] = &[
        b"access_mint",
        marketplace_key.as_ref(),
    ];

    assert_derivation(&ctx.program_id,&&ctx.accounts.access_mint.to_account_info(),  mint_seeds.clone())?;
    let mut signer_mint_seeds = mint_seeds.to_vec();
    let bump = &[params.access_mint_bump];
    signer_mint_seeds.push(bump);

    let marketplace_seeds = &[
        b"marketplace".as_ref(),
        signer_key.as_ref(),
        &[ctx.accounts.marketplace.bumps.bump],
    ];

    mint_builder(
        signer_mint_seeds,
        marketplace_seeds.to_vec(),
        vec![ExtensionType::NonTransferable],
        ctx.accounts.system_program.to_account_info(),
        ctx.accounts.token_program_2022.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.access_mint.to_account_info(),
        ctx.accounts.marketplace.to_account_info(),
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.rent.clone(),
    )?;

    (*ctx.accounts.marketplace).authority = ctx.accounts.signer.key();
    (*ctx.accounts.marketplace).token_config = TokenConfig {
        transferable: params.transferable,
    };
    (*ctx.accounts.marketplace).permission_config = PermissionConfig {
        permissionless: params.permissionless,
        access_mint: ctx.accounts.access_mint.key(),
    };
    (*ctx.accounts.marketplace).fees_config = FeesConfig {
        discount_mint: ctx.accounts.discount_mint.key(),
        fee: params.fee,
        fee_reduction: params.fee_reduction,
        fee_payer: params.fee_payer,
    };
    (*ctx.accounts.marketplace).rewards_config = RewardsConfig {
        reward_mint: ctx.accounts.reward_mint.key(),
        seller_reward: params.seller_reward,
        buyer_reward: params.buyer_reward,
        rewards_enabled: params.rewards_enabled,
    };
    (*ctx.accounts.marketplace).bumps = MarketplaceBumps {
        bump: *ctx.bumps.get("marketplace").unwrap(),
        access_mint_bump: params.access_mint_bump,
    };

    Ok(())
}
