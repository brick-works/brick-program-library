use {
    crate::{
        state::*,
        error::ErrorCode,
        utils::*,
    },
    anchor_lang::{
        prelude::*,
        system_program::System,
    },    
    anchor_spl::{
        token_interface::{MintTo, Mint, TokenInterface, TokenAccount},
        token_2022::{mint_to, ID as TokenProgram2022},
        token::{transfer, Transfer, ID as TokenProgramV0},
    },
    spl_token::native_mint::ID as NativeMint
};

#[derive(Accounts)]
pub struct RegisterBuyToken<'info> {
    pub system_program: Program<'info, System>,
    #[account(address = TokenProgramV0 @ ErrorCode::IncorrectTokenProgram, executable)]
    pub token_program_v0: Interface<'info, TokenInterface>,
    #[account(address = TokenProgram2022 @ ErrorCode::IncorrectTokenProgram, executable)]
    pub token_program_2022: Interface<'info, TokenInterface>,
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        constraint = seller.key() == product.authority
            @ ErrorCode::IncorrectAuthority
    )]
    pub seller: Option<SystemAccount<'info>>,
    #[account(
        mut,
        constraint = marketplace_auth.key() == marketplace.authority
            @ ErrorCode::IncorrectAuthority
    )]
    pub marketplace_auth: Option<SystemAccount<'info>>,
    #[account(
        mut,
        seeds = [
            b"marketplace".as_ref(),
            marketplace.authority.as_ref(),
        ],
        bump = marketplace.bumps.bump,
    )]
    pub marketplace: Box<Account<'info, Marketplace>>,
    #[account(
        mut,
        seeds = [
            b"product".as_ref(),
            product.first_id.as_ref(),
            product.second_id.as_ref(),
            marketplace.key().as_ref(),
        ],
        bump = product.bumps.bump,
    )]
    pub product: Box<Account<'info, Product>>,
    #[account(
        mut,
        seeds = [
            b"product_mint".as_ref(),
            product.key().as_ref(),
        ],
        bump = product.bumps.mint_bump,
        constraint = product_mint.key() == product.product_mint
            @ ErrorCode::IncorrectMint,
    )]    
    pub product_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        constraint = payment_mint.key() == product.seller_config.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        constraint = buyer_token_vault.owner == signer.key()
            @ ErrorCode::IncorrectAuthority,
        constraint = buyer_token_vault.mint == product.product_mint
            @ ErrorCode::IncorrectATA,
    )]
    pub buyer_token_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = buyer_transfer_vault.owner == signer.key()
            @ ErrorCode::IncorrectAuthority,
        constraint = buyer_transfer_vault.mint == payment_mint.key()
            @ ErrorCode::IncorrectATA,
    )]
    pub buyer_transfer_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(
        mut,
        constraint = seller_transfer_vault.owner == product.authority 
            @ ErrorCode::IncorrectAuthority,
        constraint = seller_transfer_vault.mint == product.seller_config.payment_mint
            @ ErrorCode::IncorrectATA,
    )]
    pub seller_transfer_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    /// ATA that receives fees
    #[account(
        mut,
        constraint = marketplace_transfer_vault.owner == marketplace.authority 
            @ ErrorCode::IncorrectAuthority,
        constraint = marketplace_transfer_vault.mint == payment_mint.key() 
            @ ErrorCode::IncorrectATA,
    )]
    pub marketplace_transfer_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    // this account holds the reward tokens
    #[account(mut)]
    pub bounty_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    // authority checked in ix logic
    #[account(
        mut,
        seeds = [
            b"reward".as_ref(),
            product.authority.as_ref(),
            marketplace.key().as_ref(),
        ],
        bump = seller_reward.bumps.bump
    )]
    pub seller_reward: Option<Account<'info, Reward>>,
    #[account(
        mut,
        constraint = seller_reward_vault.mint == payment_mint.key() 
            @ ErrorCode::IncorrectATA,
    )]
    pub seller_reward_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    // authority checked in ix logic
    #[account(
        mut,
        seeds = [
            b"reward".as_ref(),
            signer.key().as_ref(),
            marketplace.key().as_ref(),
        ],
        bump = buyer_reward.bumps.bump,
    )]
    pub buyer_reward: Option<Account<'info, Reward>>,
    #[account(
        mut,
        constraint = buyer_reward_vault.mint == payment_mint.key() 
            @ ErrorCode::IncorrectATA,
    )]
    pub buyer_reward_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
}

pub fn handler<'info>(ctx: Context<RegisterBuyToken>, amount: u32) -> Result<()> {
    let total_amount = ctx.accounts.product.seller_config.product_price
        .checked_mul(amount.into()).ok_or(ErrorCode::NumericalOverflow)?;
    let marketplace = &ctx.accounts.marketplace;

    if !marketplace.token_config.deliver_token {
        return Err(ErrorCode::IncorrectInstruction.into());
    }

    // payment and fees
    if cmp_pubkeys(&ctx.accounts.payment_mint.key(), &NativeMint) {
        let marketplace_auth = ctx.accounts.marketplace_auth.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;
        let seller = ctx.accounts.seller.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;
        
        handle_sol(
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            marketplace_auth.to_account_info(),
            seller.to_account_info(),
            marketplace.fees_config.clone(),
            ctx.accounts.product.seller_config.payment_mint,
            total_amount,
        )?;
    } else {
        let marketplace_transfer_vault = ctx.accounts.marketplace_transfer_vault.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;
        let seller_transfer_vault = ctx.accounts.seller_transfer_vault.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;        
        let buyer_transfer_vault = ctx.accounts.buyer_transfer_vault.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;

        handle_spl(
            ctx.accounts.token_program_v0.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            marketplace_transfer_vault.to_account_info(),
            seller_transfer_vault.to_account_info(),
                buyer_transfer_vault.to_account_info(),
                marketplace.fees_config.clone(),
            ctx.accounts.product.seller_config.payment_mint,
            total_amount,            
        )?;
    }

    // rewards
    if is_rewards_active(
        marketplace.rewards_config.clone(), 
        ctx.accounts.payment_mint.key(),
        ctx.program_id.key(),
    ) {
        let seller_reward = ctx.accounts.seller_reward.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;
        let buyer_reward = ctx.accounts.buyer_reward.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;

        assert_authority(&seller_reward.key(), &ctx.accounts.product.authority)?;
        assert_authority(&buyer_reward.key(), &ctx.accounts.signer.key())?;

        let seller_bonus = (marketplace.rewards_config.seller_reward as u128)
            .checked_mul(ctx.accounts.product.seller_config.product_price as u128)
            .ok_or(ErrorCode::NumericalOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::NumericalOverflow)? as u64;

        let buyer_bonus = (marketplace.rewards_config.buyer_reward as u128)
            .checked_mul(ctx.accounts.product.seller_config.product_price as u128)
            .ok_or(ErrorCode::NumericalOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::NumericalOverflow)? as u64;

        let marketplace_seeds = &[
            "marketplace".as_ref(),
            marketplace.authority.as_ref(),
            &[marketplace.bumps.bump],
        ];

        let seller_reward_vault = ctx.accounts.seller_reward_vault.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;
        let buyer_reward_vault = ctx.accounts.buyer_reward_vault.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;
        let bounty_vault = ctx.accounts.bounty_vault.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program_v0.to_account_info(), 
                Transfer {
                    from: bounty_vault.to_account_info(),
                    to: seller_reward_vault.to_account_info(),
                    authority: marketplace.to_account_info(),
                },
                &[&marketplace_seeds[..]],
            ),
            seller_bonus,
        ).map_err(|_| ErrorCode::TransferError)?;

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program_v0.to_account_info(), 
                Transfer {
                    from: bounty_vault.to_account_info(),
                    to: buyer_reward_vault.to_account_info(),
                    authority: ctx.accounts.marketplace.to_account_info()
                },
                &[&marketplace_seeds[..]],
            ),
            buyer_bonus,
        ).map_err(|_| ErrorCode::TransferError)?;
    }

    let seeds = &[
        b"product".as_ref(),
        ctx.accounts.product.first_id.as_ref(),
        ctx.accounts.product.second_id.as_ref(),
        ctx.accounts.product.marketplace.as_ref(),
        &[ctx.accounts.product.bumps.bump],
    ];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program_2022.to_account_info(),
            MintTo {
                mint: ctx.accounts.product_mint.to_account_info(),
                to: ctx.accounts.buyer_token_vault.to_account_info(),
                authority: ctx.accounts.product.to_account_info(),
            },
            &[&seeds[..]],
        ),
        amount.into()
    ).map_err(|_| ErrorCode::MintToError)?;
    
    Ok(())
}