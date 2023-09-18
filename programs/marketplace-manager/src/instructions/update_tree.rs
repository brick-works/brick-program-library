use {
    crate::state::*,
    crate::error::ErrorCode,
    anchor_lang::prelude::*,
    anchor_lang::system_program::System,
    bubblegum_cpi::{program::Bubblegum, cpi::{create_tree, accounts::CreateTree}},
    account_compression_cpi::{Noop, program::SplAccountCompression},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct UpdateProductTreeParams {
    pub max_depth: u32,
    pub max_buffer_size: u32,
}

#[derive(Accounts)]
#[instruction(args: UpdateProductTreeParams)]
pub struct UpdateProductTree<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"marketplace".as_ref(),
            signer.key().as_ref(),
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
        constraint = product.authority == signer.key()
            @ ErrorCode::IncorrectAuthority
    )]
    pub product: Box<Account<'info, Product>>,
    /// CHECK: Checked by cpi
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        bump,
        seeds::program = bubblegum_program.key()
    )]
    pub tree_authority: AccountInfo<'info>,

    /// CHECK: Checked by cpi
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,

    pub log_wrapper: Program<'info, Noop>,
    pub system_program: Program<'info, System>,
    pub bubblegum_program: Program<'info, Bubblegum>,
    pub compression_program: Program<'info, SplAccountCompression>,
}

pub fn handler(ctx: Context<UpdateProductTree>, params: UpdateProductTreeParams) -> Result<()> {
    let marketplace_key = ctx.accounts.marketplace.key();
    let product_seeds: &[&[u8]] = &[
        b"product".as_ref(),
        ctx.accounts.product.first_id.as_ref(),
        ctx.accounts.product.second_id.as_ref(),
        marketplace_key.as_ref(),
        &[ctx.accounts.product.bumps.bump],
    ];

    create_tree(
        CpiContext::new_with_signer(
            ctx.accounts.bubblegum_program.to_account_info().clone(),
            CreateTree {
                tree_authority: ctx.accounts.tree_authority.to_account_info().clone(),
                merkle_tree: ctx.accounts.merkle_tree.to_account_info().clone(),
                payer: ctx.accounts.payer.to_account_info().clone(),
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
    ctx.accounts.product.merkle_tree = ctx.accounts.merkle_tree.key();
    
    Ok(())
}
