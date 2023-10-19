use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{
        token_interface::{Mint, TokenInterface, TokenAccount, mint_to, MintTo},
        metadata::{
            CreateMetadataAccountsV3,
            create_metadata_accounts_v3,
            mpl_token_metadata::types::{DataV2, Creator, CollectionDetails},
            create_master_edition_v3, 
            CreateMasterEditionV3, 
            ID as TokenMetadataProgram
        },
        associated_token::AssociatedToken
    }
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitRolesParams {
    pub service_collection_uri: String,
    pub council_collection_uri: String,
}

#[derive(Accounts)]
pub struct InitRoles<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"network".as_ref()],
        bump = network.bump
    )]
    pub network: Account<'info, Network>,
    #[account(
        mut,
        seeds = [
            b"council".as_ref(),
            network.key().as_ref(),
        ],
        bump = network.council_collection_bump,
    )]
    pub council_collection: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = council_collection,
        associated_token::authority = network,
        associated_token::token_program = token_program,
    )]
    pub council_collection_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(), 
            token_metadata_program.key().as_ref(), 
            council_collection.key().as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub council_collection_metadata: UncheckedAccount<'info>,
    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(),
            token_metadata_program.key().as_ref(), 
            council_collection.key().as_ref(), 
            "edition".as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub council_collection_master_edition: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [
            b"service".as_ref(),
            network.key().as_ref(),
        ],
        bump = network.service_collection_bump,
    )]
    pub service_collection: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = service_collection,
        associated_token::authority = network,
        associated_token::token_program = token_program
    )]
    pub service_collection_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(), 
            token_metadata_program.key().as_ref(), 
            service_collection.key().as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub service_collection_metadata: UncheckedAccount<'info>,
    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(),
            token_metadata_program.key().as_ref(), 
            service_collection.key().as_ref(), 
            "edition".as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub service_collection_master_edition: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK:
    #[account(address = TokenMetadataProgram)]
    pub token_metadata_program: AccountInfo<'info>,
}

pub fn handler<'info>(ctx: Context<InitRoles>, params: InitRolesParams) -> Result<()> {
    let network_seeds = &[
        b"network".as_ref(),
        &[ctx.accounts.network.bump],
    ];

    mint_to(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.to_account_info(),
          MintTo {
            mint: ctx.accounts.council_collection.to_account_info(),
            to: ctx.accounts.council_collection_vault.to_account_info(),
            authority: ctx.accounts.network.to_account_info(),
          },
          &[&network_seeds[..]],
        ),
        1,
    )?;

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.council_collection_metadata.to_account_info().clone(),
                mint: ctx.accounts.council_collection.to_account_info().clone(),
                mint_authority: ctx.accounts.network.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                update_authority: ctx.accounts.network.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
            &[&network_seeds[..]],
        ),
        DataV2 {
            name: "Council member".to_string(),
            symbol: "DAO".to_string(),
            uri: params.council_collection_uri,
            seller_fee_basis_points: 0,
            creators: Some(Vec::from([Creator {
                address: ctx.accounts.signer.key(),
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
                edition: ctx.accounts.council_collection_master_edition.to_account_info().clone(),
                mint: ctx.accounts.council_collection.to_account_info().clone(),
                update_authority: ctx.accounts.network.to_account_info().clone(),
                mint_authority: ctx.accounts.network.to_account_info().clone(),
                metadata: ctx.accounts.council_collection_metadata.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                token_program: ctx.accounts.token_program.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
        &[&network_seeds[..]],
        ),
        Some(0),
    )?;

    mint_to(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.to_account_info(),
          MintTo {
            mint: ctx.accounts.service_collection.to_account_info(),
            to: ctx.accounts.service_collection_vault.to_account_info(),
            authority: ctx.accounts.network.to_account_info(),
          },
          &[&network_seeds[..]],
        ),
        1,
    )?;

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.service_collection_metadata.to_account_info().clone(),
                mint: ctx.accounts.service_collection.to_account_info().clone(),
                mint_authority: ctx.accounts.network.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                update_authority: ctx.accounts.network.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
            &[&network_seeds[..]],
        ),
        DataV2 {
            name: "Service provider".to_string(),
            symbol: "DAO".to_string(),
            uri: params.service_collection_uri,
            seller_fee_basis_points: 0,
            creators: Some(Vec::from([Creator {
                address: ctx.accounts.signer.key(),
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
                edition: ctx.accounts.service_collection_master_edition.to_account_info().clone(),
                mint: ctx.accounts.service_collection.to_account_info().clone(),
                update_authority: ctx.accounts.network.to_account_info().clone(),
                mint_authority: ctx.accounts.network.to_account_info().clone(),
                metadata: ctx.accounts.service_collection_metadata.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                token_program: ctx.accounts.token_program.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
        &[&network_seeds[..]],
        ),
        Some(0),
    )?;

    Ok(())
}
