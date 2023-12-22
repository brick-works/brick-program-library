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
        instruction::initialize_non_transferable_mint,
        
    },
};

pub fn mint_builder<'info>(
    mint_seeds: &[&[u8]],
    mint_authority_seeds: &[&[u8]; 3],
    extensions: Vec<ExtensionType>,
    mint: AccountInfo<'info>,
    mint_authority: AccountInfo<'info>,
    signer: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>,
    system_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
) -> std::result::Result<(), ErrorCode> {
    let space = if extensions.is_empty() {
        ExtensionType::try_calculate_account_len::<Mint2022>(&[]).unwrap_or_default()
    } else {
        extensions
            .iter()
            .map(|ext| ExtensionType::try_calculate_account_len::<Mint2022>(&[*ext]).unwrap_or_default())
            .sum()
    };

    create_account(
        CpiContext::new_with_signer(
            system_program,
            CreateAccount {
                from: signer.clone(),
                to: mint.clone(),
            },
            &[&mint_seeds[..]],
        ),
        rent.minimum_balance(space),
        space as u64,
        &token_program.key(),
    )
    .map_err(|_| ErrorCode::CreateAccountError)?;

    for ext in extensions {
        match ext {
            ExtensionType::NonTransferable => {
                invoke_signed(
                    &initialize_non_transferable_mint(
                        &token_program.key(),
                        &mint.key(),
                    )
                    .map_err(|_| ErrorCode::TransferError)?,
                    &[mint_authority.clone(), mint.clone()],
                    &[&mint_seeds[..]],
                )
                .map_err(|_| ErrorCode::MintExtensionError)?;
            }
            _ => {
                return Err(ErrorCode::ExtensionNotSupported.into());
            }
        }
    }

    initialize_mint(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            InitializeMint {
                mint,
                rent: rent.to_account_info(),
            },
            &[&mint_seeds[..], &mint_authority_seeds[..]],
        ),
        0,
        &mint_authority.key(),
        None,
    )
    .map_err(|_| ErrorCode::InitMintError)?;

    Ok(())
}
