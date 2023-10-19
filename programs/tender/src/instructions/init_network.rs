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

#[derive(Accounts)]
pub struct InitNetwork<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        space = NETWORK_SIZE,
        payer = signer,
        seeds = [b"network".as_ref()],
        bump
    )]
    pub network: Account<'info, Network>,
    /// CHECK: account init during the instruction logic
    #[account(
        init,
        payer = signer,
        mint::decimals = 6, // like USDC
        mint::authority = network,
        mint::freeze_authority = network,
        mint::token_program = token_program,
        seeds = [
            b"network_mint".as_ref(),
            network.key().as_ref(),
        ],
        bump,
    )]
    pub network_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = network,
        mint::freeze_authority = network,
        mint::token_program = token_program,
        seeds = [
            b"council".as_ref(),
            network.key().as_ref(),
        ],
        bump,
    )]
    pub council_collection: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = network,
        mint::freeze_authority = network,
        mint::token_program = token_program,
        seeds = [
            b"service".as_ref(),
            network.key().as_ref(),
        ],
        bump,
    )]
    pub service_collection: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = network,
        mint::freeze_authority = network,
        mint::token_program = token_program,
        seeds = [
            b"proposal".as_ref(),
            network.key().as_ref(),
        ],
        bump,
    )]
    pub proposal_collection: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = proposal_collection,
        associated_token::authority = network,
        associated_token::token_program = token_program
    )]
    pub proposal_collection_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(), 
            token_metadata_program.key().as_ref(), 
            proposal_collection.key().as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub proposal_metadata: UncheckedAccount<'info>,
    /// CHECK: this will be verified by token metadata program
    #[account(
        mut,
        seeds = [
            b"metadata".as_ref(),
            token_metadata_program.key().as_ref(),
            proposal_collection.key().as_ref(),
            b"edition".as_ref(),
        ],
        bump,
        seeds::program = token_metadata_program.key()
    )]
    pub proposal_collection_master_edition: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK:
    #[account(address = TokenMetadataProgram)]
    pub token_metadata_program: AccountInfo<'info>,
}

pub fn handler<'info>(ctx: Context<InitNetwork>, proposal_collection_uri: String) -> Result<()> {
    (*ctx.accounts.network).council_collection = ctx.accounts.council_collection.key();
    (*ctx.accounts.network).service_collection = ctx.accounts.service_collection.key();
    (*ctx.accounts.network).proposal_collection = ctx.accounts.proposal_collection.key();
    (*ctx.accounts.network).network_mint = ctx.accounts.network_mint.key();
    (*ctx.accounts.network).council_collection_bump = ctx.bumps.council_collection;
    (*ctx.accounts.network).service_collection_bump = ctx.bumps.service_collection;
    (*ctx.accounts.network).proposal_collection_bump = ctx.bumps.proposal_collection;
    (*ctx.accounts.network).mint_bump = ctx.bumps.network_mint;
    (*ctx.accounts.network).bump = ctx.bumps.network;

    let network_seeds = &[
        b"network".as_ref(),
        &[ctx.accounts.network.bump],
    ];

    mint_to(
        CpiContext::new_with_signer(
          ctx.accounts.token_program.to_account_info(),
          MintTo {
            mint: ctx.accounts.proposal_collection.to_account_info(),
            to: ctx.accounts.proposal_collection_vault.to_account_info(),
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
                metadata: ctx.accounts.proposal_metadata.to_account_info().clone(),
                mint: ctx.accounts.proposal_collection.to_account_info().clone(),
                mint_authority: ctx.accounts.network.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                update_authority: ctx.accounts.network.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
            &[&network_seeds[..]],
        ),
        DataV2 {
            name: "Proposal".to_string(),
            symbol: "DAO".to_string(),
            uri: proposal_collection_uri,
            seller_fee_basis_points: 0,
            creators: Some(Vec::from([Creator {
                address: ctx.accounts.signer.key(),
                verified: false,
                share: 100,
            }])),
            collection: None,
            uses: None,
        },
        false,
        false,
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.proposal_collection_master_edition.to_account_info().clone(),
                mint: ctx.accounts.proposal_collection.to_account_info().clone(),
                update_authority: ctx.accounts.network.to_account_info().clone(),
                mint_authority: ctx.accounts.network.to_account_info().clone(),
                metadata: ctx.accounts.proposal_metadata.to_account_info().clone(),
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
