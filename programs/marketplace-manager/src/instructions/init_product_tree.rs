use {
    crate::state::*,
    crate::utils::assert_derivation,
    crate::error::ErrorCode,
    crate::utils::{create_metadata_accounts_v3, CreateMetadataAccountsV3, create_master_edition_v3, CreateMasterEditionV3},
    anchor_lang::prelude::*,
    anchor_lang::system_program::System,
    anchor_spl::{token_interface::{Mint, TokenAccount, TokenInterface, mint_to, MintTo}, associated_token::AssociatedToken},
    bubblegum_cpi::{program::Bubblegum, cpi::{create_tree, accounts::CreateTree}},
    account_compression_cpi::{Noop, program::SplAccountCompression},
    mpl_token_metadata::state::{DataV2, Creator, CollectionDetails}
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitProductTreeParams {
    pub id: [u8; 16],
    pub product_price: u64,
    pub max_depth: u32,
    pub max_buffer_size: u32,
    pub name: String,
    pub metadata_url: String,
    pub fee_basis_points: u16,
}

#[derive(Accounts)]
#[instruction(params: InitProductTreeParams)]
pub struct InitProductTree<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
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
        init,
        payer = signer,
        space = PRODUCT_SIZE,
        seeds = [
            b"product".as_ref(),
            params.id.as_ref(),
        ],
        bump,
    )]
    pub product: Box<Account<'info, Product>>,
    /// CHECK: This account is init in this instruction
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = product,
        mint::freeze_authority = product,
        mint::token_program = token_program,
        seeds = [
            b"product_mint".as_ref(),
            product.key().as_ref(),
        ],
        bump,
    )]
    pub product_mint: Box<InterfaceAccount<'info, Mint>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"access_mint".as_ref(),
            marketplace.key().as_ref(),
        ],
        bump = marketplace.bumps.access_mint_bump,
        constraint = access_mint.key() == marketplace.permission_config.access_mint
            @ ErrorCode::IncorrectMint
    )]    
    pub access_mint: Option<Box<InterfaceAccount<'info, Mint>>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = product_mint,
        associated_token::authority = product,
        associated_token::token_program = token_program
    )]
    pub product_mint_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = access_vault.mint == marketplace.permission_config.access_mint
            @ ErrorCode::IncorrectMint,
        constraint = access_vault.owner == signer.key()
            @ ErrorCode::IncorrectAuthority
    )]    
    pub access_vault: Option<Box<InterfaceAccount<'info, TokenAccount>>>,
    /// CHECK: Handled by cpi
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
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: Checked by cpi
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        bump,
        seeds::program = bubblegum_program.key()
    )]
    pub tree_authority: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: Checked with constraints
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: AccountInfo<'info>,
    pub log_wrapper: Program<'info, Noop>,
    pub system_program: Program<'info, System>,
    pub bubblegum_program: Program<'info, Bubblegum>,
    pub compression_program: Program<'info, SplAccountCompression>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler<'info>(ctx: Context<InitProductTree>, params: InitProductTreeParams) -> Result<()> {
    if !ctx.accounts.marketplace.permission_config.permissionless {
        let access_vault = ctx.accounts.access_vault.as_ref()
            .ok_or(ErrorCode::OptionalAccountNotProvided)?;

        if access_vault.amount == 0 {
            return Err(ErrorCode::NotInWithelist.into());
        }
    }

    let product_key = ctx.accounts.product.key();

    (*ctx.accounts.product).authority = ctx.accounts.signer.key();
    (*ctx.accounts.product).id = params.id;
    (*ctx.accounts.product).merkle_tree = ctx.accounts.merkle_tree.key();
    (*ctx.accounts.product).product_mint = ctx.accounts.product_mint.key();
    (*ctx.accounts.product).seller_config = SellerConfig {
        payment_mint: ctx.accounts.payment_mint.key(),
        product_price: params.product_price,
    };
    (*ctx.accounts.product).bumps = ProductBumps {
        bump: *ctx.bumps.get("product").unwrap(),
        mint_bump: *ctx.bumps.get("product_mint").unwrap(),
    };

    let mint_seeds: &[&[u8]] = &[
        b"product_mint",
        product_key.as_ref(),
    ];

    assert_derivation(&ctx.program_id,&ctx.accounts.product_mint.to_account_info(),  mint_seeds.clone())?;
    let mut signer_mint_seeds = mint_seeds.to_vec();
    let bump = &[*ctx.bumps.get("product_mint").unwrap()];
    signer_mint_seeds.push(bump);

    let product_seeds = &[
        b"product".as_ref(),
        ctx.accounts.product.id.as_ref(),
        &[ctx.accounts.product.bumps.bump],
    ];

    mint_to(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.to_account_info(),
          MintTo {
            mint: ctx.accounts.product_mint.to_account_info(),
            to: ctx.accounts.product_mint_vault.to_account_info(),
            authority: ctx.accounts.product.to_account_info(),
          },
          &[&product_seeds[..]],
        ),
        1,
    )?;

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata.to_account_info().clone(),
                mint: ctx.accounts.product_mint.to_account_info().clone(),
                mint_authority: ctx.accounts.product.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                update_authority: ctx.accounts.product.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
            &[&product_seeds[..]],
        ),
        DataV2 {
            name: params.name.clone(),
            symbol: "BRICK".to_string(),
            uri: params.metadata_url,
            seller_fee_basis_points: 0,
            creators: Some(Vec::from([Creator {
                address: ctx.accounts.product.authority,
                verified: false,
                share: 100,
            }])),
            collection: None,
            uses: None,
        },
        true,
        true,
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition.to_account_info().clone(),
                mint: ctx.accounts.product_mint.to_account_info().clone(),
                update_authority: ctx.accounts.product.to_account_info().clone(),
                mint_authority: ctx.accounts.product.to_account_info().clone(),
                metadata: ctx.accounts.metadata.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                token_program: ctx.accounts.token_program.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
        &[&product_seeds[..]],
        ),
        Some(0),
    )?;

    create_tree(
        CpiContext::new_with_signer(
          ctx.accounts.bubblegum_program.to_account_info().clone(),
          CreateTree {
            tree_authority: ctx.accounts.tree_authority.to_account_info().clone(),
            merkle_tree: ctx.accounts.merkle_tree.to_account_info().clone(),
            payer: ctx.accounts.signer.to_account_info().clone(),
            tree_creator: ctx.accounts.product.to_account_info().clone(),
            log_wrapper: ctx.accounts.log_wrapper.to_account_info().clone(),
            compression_program: ctx.accounts.compression_program.to_account_info().clone(),
            system_program: ctx.accounts.system_program.to_account_info().clone(),
          },
          &[&product_seeds[..]],
        ),
        params.max_depth,
        params.max_buffer_size,
        None,
    )?;
  
    Ok(())
}
