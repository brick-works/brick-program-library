use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{
        token_interface::{Mint, TokenInterface, TokenAccount, mint_to, MintTo},
        metadata::{
            CreateMetadataAccountsV3,
            create_metadata_accounts_v3,
            mpl_token_metadata::types::{DataV2, Creator, Collection, CollectionDetails},
            create_master_edition_v3, 
            CreateMasterEditionV3, 
            ID as TokenMetadataProgram,
            VerifySizedCollectionItem, 
            verify_sized_collection_item,
        },
        associated_token::AssociatedToken
    }
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitProposalParams {
    id: [u8; 16],
    name: String,
    proposal_uri: String
}

#[derive(Accounts)]
#[instruction(params: InitProposalParams)]
pub struct InitProposal<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"network".as_ref()],
        bump = network.bump
    )]
    pub network: Account<'info, Network>,
    #[account(
        init,
        space = PROPOSAL_SIZE,
        payer = signer,
        seeds = [
            b"proposal".as_ref(),
            params.id.as_ref(),
        ],
        bump
    )]
    pub proposal: Box<Account<'info, Proposal>>,
    #[account(
        init,
        payer = signer,
        seeds = [
            b"vault".as_ref(),
            proposal.key().as_ref(),
        ],
        bump,
        token::mint = payment_mint,
        token::authority = proposal,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub payment_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [
            b"proposal".as_ref(),
            network.key().as_ref(),
        ],
        bump,
    )]
    pub proposal_collection: Box<InterfaceAccount<'info, Mint>>,
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
    pub proposal_collection_metadata: UncheckedAccount<'info>,
    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(),
            token_metadata_program.key().as_ref(), 
            proposal_collection.key().as_ref(), 
            "edition".as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub proposal_collection_master_edition: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = proposal,
        mint::freeze_authority = proposal,
        mint::token_program = token_program,
        seeds = [
            b"proposal_mint".as_ref(),
            proposal.key().as_ref(),
        ],
        bump
    )]
    pub proposal_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(),
            token_metadata_program.key().as_ref(), 
            proposal_mint.key().as_ref(), 
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub proposal_metadata: UncheckedAccount<'info>,
    /// CHECK: Handled by cpi
    #[account(
        mut,
        seeds = [
            "metadata".as_ref(),
            token_metadata_program.key().as_ref(), 
            proposal_mint.key().as_ref(), 
            "edition".as_ref()
        ],
        seeds::program = token_metadata_program.key(),
        bump,
    )]
    pub proposal_master_edition: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = proposal_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub user_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Checked with constraints
    #[account(address = TokenMetadataProgram)]
    pub token_metadata_program: AccountInfo<'info>,
}

pub fn handler<'info>(ctx: Context<InitProposal>, params: InitProposalParams) -> Result<()> {
    let signer_key: Pubkey = ctx.accounts.signer.key();

    (*ctx.accounts.proposal).id = params.id;
    (*ctx.accounts.proposal).authority = signer_key;
    (*ctx.accounts.proposal).vault = ctx.accounts.vault.key();
    (*ctx.accounts.proposal).state = RequestState::SigningOff;
    (*ctx.accounts.proposal).vault_bump = ctx.bumps.vault;
    (*ctx.accounts.proposal).bump = ctx.bumps.proposal;

    let proposal_seeds = &[
        b"proposal".as_ref(),
        ctx.accounts.proposal.id.as_ref(),
        &[ctx.accounts.proposal.bump],
    ];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.proposal_mint.to_account_info(),
                to: ctx.accounts.user_vault.to_account_info(),
                authority: ctx.accounts.proposal.to_account_info(),
            },
            &[&proposal_seeds[..]],
        ),
        1
    )?;

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.proposal_metadata.to_account_info().clone(),
                mint: ctx.accounts.proposal_mint.to_account_info().clone(),
                mint_authority: ctx.accounts.proposal.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                update_authority: ctx.accounts.proposal.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
            &[&proposal_seeds[..]],
        ),
        DataV2 {
            name: params.name,
            symbol: "PRO".to_string(),
            uri: params.proposal_uri,
            seller_fee_basis_points: 0,
            creators: Some(Vec::from([Creator {
                address: ctx.accounts.signer.key(),
                verified: false,
                share: 100,
            }])),
            collection: Some(Collection {
                key: ctx.accounts.proposal_collection.key(),
                verified: false
            }),
            uses: None,
        },
        false,
        true,
        Some(CollectionDetails::V1 { size: 0 }),
    )?;

    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.proposal_master_edition.to_account_info().clone(),
                mint: ctx.accounts.proposal_mint.to_account_info().clone(),
                update_authority: ctx.accounts.proposal.to_account_info().clone(),
                mint_authority: ctx.accounts.proposal.to_account_info().clone(),
                metadata: ctx.accounts.proposal_metadata.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                token_program: ctx.accounts.token_program.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
        &[&proposal_seeds[..]],
        ),
        Some(1),
    )?;

    let network_seeds = &[
        b"network".as_ref(),
        &[ctx.accounts.network.bump],
    ];

    verify_sized_collection_item(CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.clone(),
        VerifySizedCollectionItem {
            metadata: ctx.accounts.proposal_metadata.to_account_info().clone(),
            collection_authority: ctx.accounts.network.to_account_info().clone(),
            payer: ctx.accounts.signer.to_account_info().clone(),
            collection_mint: ctx.accounts.proposal_collection.to_account_info().clone(),
            collection_metadata: ctx.accounts.proposal_collection_metadata.to_account_info().clone(),
            collection_master_edition: ctx.accounts.proposal_collection_master_edition.to_account_info().clone(),
        },
        &[&network_seeds[..]],
        ),
        None
    )?;    
    
    Ok(())
}
