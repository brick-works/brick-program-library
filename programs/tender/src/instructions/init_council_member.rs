use {
    crate::state::*,
    anchor_lang::prelude::*,
    anchor_spl::{
        token_interface::{Mint, TokenInterface, TokenAccount, mint_to, MintTo},
        metadata::{
            create_master_edition_v3, 
            CreateMasterEditionV3, 
            VerifySizedCollectionItem, 
            verify_sized_collection_item,
            ID as TokenMetadataProgram
        }
    }
};

#[derive(Accounts)]
pub struct InitCouncilMember<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub receiver: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"network".as_ref()],
        bump = network.bump
    )]
    pub network: Account<'info, Network>,
    #[account(
        mut,
        seeds = [
            b"proposal".as_ref(),
            proposal.id.as_ref()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(
        mut,
        seeds = [
            b"council".as_ref(),
            network.key().as_ref(),
        ],
        bump,
    )]
    pub council_collection: Box<InterfaceAccount<'info, Mint>>,
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
    pub council_metadata: UncheckedAccount<'info>,
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
    pub council_master_edition: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        token::mint = council_master_edition,
        token::authority = receiver,
    )]
    pub receiver_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Checked with constraints
    #[account(address = TokenMetadataProgram)]
    pub token_metadata_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler<'info>(ctx: Context<InitCouncilMember>) -> Result<()> {
    let network_seeds = &[
        b"network".as_ref(),
        &[ctx.accounts.network.bump],
    ];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.council_master_edition.to_account_info(),
                to: ctx.accounts.receiver_vault.to_account_info(),
                authority: ctx.accounts.network.to_account_info(),
            },
            &[&network_seeds[..]],
        ),
        1
    )?;

    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.council_master_edition.to_account_info().clone(),
                mint: ctx.accounts.council_collection.to_account_info().clone(),
                update_authority: ctx.accounts.network.to_account_info().clone(),
                mint_authority: ctx.accounts.network.to_account_info().clone(),
                metadata: ctx.accounts.council_metadata.to_account_info().clone(),
                payer: ctx.accounts.signer.to_account_info().clone(),
                token_program: ctx.accounts.token_program.to_account_info().clone(),
                system_program: ctx.accounts.system_program.to_account_info().clone(),
                rent: ctx.accounts.rent.to_account_info().clone(),
            },
        &[&network_seeds[..]],
        ),
        Some(1),
    )?;

    verify_sized_collection_item(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.clone(),
            VerifySizedCollectionItem {
                payer: ctx.accounts.signer.to_account_info().clone(),
                metadata: ctx.accounts.council_metadata.to_account_info().clone(),
                collection_authority: ctx.accounts.network.to_account_info().clone(),
                collection_mint: ctx.accounts.council_collection.to_account_info().clone(),
                collection_metadata: ctx.accounts.council_collection_metadata.to_account_info().clone(),
                collection_master_edition: ctx.accounts.council_collection_master_edition.to_account_info().clone(),
            },
            &[&network_seeds[..]],
        ),
        None    
    )?;    

    Ok(())
}
