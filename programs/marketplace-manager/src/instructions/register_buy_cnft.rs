use {
    crate::{
        utils::*, 
        state::*,
        error::ErrorCode,
    },
    anchor_lang::{
        prelude::*,
        system_program::System,
    },    
    anchor_spl::{
        token_interface::{Mint, TokenInterface, TokenAccount},
        token::{transfer, Transfer, ID as TokenProgramV0},
    },
    spl_token::native_mint::ID as NativeMint,
    bubblegum_cpi::{
        cpi::{accounts::MintToCollectionV1, mint_to_collection_v1},
        program::Bubblegum,
        Collection, Creator, MetadataArgs, TokenProgramVersion, TokenStandard, TreeConfig,
    },
    account_compression_cpi::{program::SplAccountCompression, Noop}
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterBuyCnftParams {
    pub amount: u32,
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Accounts)]
pub struct RegisterBuyCnft<'info> {
    pub system_program: Program<'info, System>,
    #[account(address = TokenProgramV0 @ ErrorCode::IncorrectTokenProgram, executable)]
    pub token_program_v0: Interface<'info, TokenInterface>,
    pub rent: Sysvar<'info, Rent>,
    pub log_wrapper: Program<'info, Noop>,
    pub bubblegum_program: Program<'info, Bubblegum>,
    pub compression_program: Program<'info, SplAccountCompression>,
    /// CHECK: Checked with constraints
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,
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
            product.marketplace.as_ref(),
        ],
        bump = product.bumps.bump,
    )]
    pub product: Box<Account<'info, Product>>,
    #[account(
        constraint = payment_mint.key() == product.seller_config.payment_mint
            @ ErrorCode::IncorrectMint,
    )]
    pub payment_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [
            b"product_mint".as_ref(),
            product.key().as_ref(),
        ],
        bump = product.bumps.mint_bump
    )]
    pub product_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        constraint = buyer_transfer_vault.owner == signer.key()
            @ ErrorCode::IncorrectAuthority,
        constraint = buyer_transfer_vault.mint == product.seller_config.payment_mint.key()
            @ ErrorCode::IncorrectATA,
    )]
    pub buyer_transfer_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(
        mut,
        constraint = seller_transfer_vault.owner == product.authority 
            @ ErrorCode::IncorrectAuthority,
        constraint = seller_transfer_vault.mint == product.seller_config.payment_mint.key()
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
    // the validation of these accounts is done in the ix logic
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
    #[account(mut)]
    pub seller_reward_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    #[account(
        mut,
        seeds = [
            b"reward".as_ref(),
            signer.key().as_ref(),
            marketplace.key().as_ref(),
        ],
        bump = buyer_reward.bumps.bump
    )]
    pub buyer_reward: Option<Account<'info, Reward>>,
    #[account(mut)]
    pub buyer_reward_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,

    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(), 
            token_metadata_program.key().as_ref(), 
            product_mint.key().as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub metadata: UncheckedAccount<'info>,
    /// CHECK: Checked by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(),
            token_metadata_program.key().as_ref(), 
            product_mint.key().as_ref(), 
            "edition".as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub master_edition: UncheckedAccount<'info>,

    /// CHECK: Checked by cpi
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        seeds::program = bubblegum_program.key(),
        bump,
    )]
    pub tree_authority: Box<Account<'info, TreeConfig>>,
    /// CHECK: cpi
    #[account(
        seeds = ["collection_cpi".as_bytes()],
        seeds::program = bubblegum_program.key(),
        bump,
    )]
    pub bubblegum_signer: UncheckedAccount<'info>,
    /// CHECK: Checked by cpi
    #[account(mut)]
    pub merkle_tree: AccountInfo<'info>,
}

pub fn handler<'info>(ctx: Context<RegisterBuyCnft>, params: RegisterBuyCnftParams) -> Result<()> {
    let total_amount = ctx.accounts.product.seller_config.product_price
        .checked_mul(params.amount.into()).ok_or(ErrorCode::NumericalOverflow)?;
    let marketplace = &ctx.accounts.marketplace;
    let marketplace_key = ctx.accounts.marketplace.key();

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
        
        assert_authority(&seller_reward.authority, &ctx.accounts.product.authority)?;
        assert_authority(&buyer_reward.authority, &ctx.accounts.signer.key())?;

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
                    authority: marketplace.to_account_info()
                },
                &[&marketplace_seeds[..]],
            ),
            buyer_bonus,
        ).map_err(|_| ErrorCode::TransferError)?;
    }

    let product_seeds = &[
        b"product".as_ref(),
        ctx.accounts.product.first_id.as_ref(),
        ctx.accounts.product.second_id.as_ref(),
        marketplace_key.as_ref(),
        &[ctx.accounts.product.bumps.bump],
    ];

    mint_to_collection_v1(
        CpiContext::new_with_signer(
            ctx.accounts.bubblegum_program.to_account_info(),
            MintToCollectionV1 {
                bubblegum_signer: ctx.accounts.bubblegum_signer.to_account_info(),
                collection_authority: ctx.accounts.product.to_account_info(),
                collection_mint: ctx.accounts.product_mint.to_account_info(),
                collection_authority_record_pda: ctx.accounts.bubblegum_program.to_account_info(),
                collection_metadata: ctx.accounts.metadata.to_account_info(),
                compression_program: ctx.accounts.compression_program.to_account_info(),
                edition_account: ctx.accounts.master_edition.to_account_info(),
                leaf_delegate: ctx.accounts.signer.to_account_info(),
                leaf_owner: ctx.accounts.signer.to_account_info(),
                log_wrapper: ctx.accounts.log_wrapper.to_account_info(),
                merkle_tree: ctx.accounts.merkle_tree.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
                tree_authority: ctx.accounts.tree_authority.to_account_info(),
                tree_delegate: ctx.accounts.product.to_account_info(),
            },
            &[&product_seeds[..]],
        ), MetadataArgs {
            name: params.name,
            symbol: params.symbol,
            uri: params.uri,
            seller_fee_basis_points: 0,
            creators: Vec::from([Creator {
                address: ctx.accounts.product.authority,
                verified: false,
                share: 100,
            }]),
            collection: Some(Collection {
                key: ctx.accounts.product_mint.key(),
                verified: false,
            }),
            is_mutable: true,
            primary_sale_happened: true,
            edition_nonce: None,
            token_program_version: TokenProgramVersion::Original,
            token_standard: Some(TokenStandard::NonFungible),
            uses: None
        }
    )?;

    Ok(())
}

