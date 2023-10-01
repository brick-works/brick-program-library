use {
    anchor_lang::prelude::*,
    crate::error::ErrorCode,
    anchor_lang::{
        system_program::{CreateAccount, create_account},
        solana_program::program::invoke_signed,
    },
    anchor_spl::token_interface::{
        InitializeMint, 
        initialize_mint,
    },
    spl_token_2022::{
        extension::ExtensionType,
        state::Mint as Mint2022,
        instruction::initialize_non_transferable_mint
    },
};

pub fn mint_builder<'info>(
    mint_seeds: Vec<&[u8]>,
    mint_authority_seeds: Vec<&[u8]>,
    system_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    rent_info: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    mint_authority: AccountInfo<'info>,
    signer: AccountInfo<'info>,
    rent: Sysvar<'_, Rent>,
) -> std::result::Result<(), ErrorCode> {
    let space = ExtensionType::get_account_len::<Mint2022>(&[ExtensionType::NonTransferable]);

    create_account(
        CpiContext::new_with_signer(
            system_program,
            CreateAccount { 
                from: signer, 
                to: mint.clone()
            },
            &[&mint_seeds[..]],
        ),
        rent.minimum_balance(space), 
        space as u64, 
        &token_program.key()
    ).map_err(|_| ErrorCode::CreateAccountError)?;

    invoke_signed(
        &initialize_non_transferable_mint(
            &token_program.key(), 
            &mint.key().clone()
        ).map_err(|_| ErrorCode::ErrorInitNotTransferable)?,
        &[
            mint_authority.clone(),
            mint.clone()
        ],
        &[&mint_seeds[..]],
    ).map_err(|_| ErrorCode::InitMintError)?; 

    initialize_mint(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            InitializeMint {
                mint,
                rent: rent_info,
            },
            &[&mint_seeds[..], &mint_authority_seeds[..]],
        ),
        0, 
        &mint_authority.key(), 
        None
    ).map_err(|_| ErrorCode::InitMintError)?;

    Ok(())
}
