use {
    super::cmp_pubkeys,
    crate::error::ErrorCode,
    crate::state::*,
    anchor_lang::{
        prelude::*,
        system_program::{
            transfer as native_transfer,
            Transfer as NativeTransfer
        },
    },    
    anchor_spl::token::{transfer, Transfer},
};

pub fn handle_sol<'info>(
    system_program: AccountInfo<'info>,
    signer: AccountInfo<'info>,
    marketplace_auth: AccountInfo<'info>,
    seller: AccountInfo<'info>,
    fees_config: FeesConfig,
    payment_mint: Pubkey,
    total_payment: u64,
) -> Result<()> {
    if fees_config.fee > 0 {
        let (total_fee, seller_amount) = calculate_transfer_distribution(
            fees_config,
            payment_mint,
            total_payment,
        )?;

        native_transfer(
            CpiContext::new(
                system_program.clone(), 
                NativeTransfer {
                    from: signer.clone(),
                    to: marketplace_auth,
            }), 
            total_fee
        )?;

        native_transfer(
            CpiContext::new(
                system_program, 
                NativeTransfer {
                    from: signer,
                    to: seller,
                }
            ), 
            seller_amount
        )?;
    } else {
        native_transfer(
            CpiContext::new(
                system_program, 
                NativeTransfer {
                    from: signer,
                    to: seller,
                }
            ), 
            total_payment
        )?;
    }

    Ok(())
}

pub fn handle_spl<'info>(
    token_program_v0: AccountInfo<'info>,
    signer: AccountInfo<'info>,
    marketplace_transfer_vault: AccountInfo<'info>,
    seller_transfer_vault: AccountInfo<'info>,
    buyer_transfer_vault: AccountInfo<'info>,
    fees_config: FeesConfig,
    payment_mint: Pubkey,
    total_payment: u64,
) -> Result<()> {
    if fees_config.fee > 0 {
        let (total_fee, seller_amount) = calculate_transfer_distribution(
            fees_config,
            payment_mint,
            total_payment,
        )?;

        transfer(
            CpiContext::new(
                token_program_v0.clone(), 
                Transfer {
                    from: buyer_transfer_vault.clone(),
                    to: marketplace_transfer_vault,
                    authority: signer.clone(),
                },
            ),
            total_fee,
        ).map_err(|_| ErrorCode::TransferError)?;

        transfer(
            CpiContext::new(
                token_program_v0, 
                Transfer {
                    from: buyer_transfer_vault,
                    to: seller_transfer_vault,
                    authority: signer,
                },
            ),
            seller_amount,
        ).map_err(|_| ErrorCode::TransferError)?;
    } else {
        transfer(
            CpiContext::new(
                token_program_v0, 
                Transfer {
                    from: buyer_transfer_vault,
                    to: seller_transfer_vault,
                    authority: signer,
                },
            ),
            total_payment,
        ).map_err(|_| ErrorCode::TransferError)?;
    }

    Ok(())
}

/// Calculates the distribution of the token amount, considering transaction fee and potential fee reduction.
/// Adjusts the fee if the payment mint is the same as the reward mint.
/// Also is considered fee_payer decided by the marketplace.
pub fn calculate_transfer_distribution(
    fees: FeesConfig,
    payment_mint: Pubkey, 
    amount: u64,
) -> std::result::Result<(u64, u64), ErrorCode> {
    let adjusted_fee_basis_points: u16 = if cmp_pubkeys(&payment_mint, &fees.discount_mint) {
        fees.fee.saturating_sub(fees.fee_reduction)
    } else {
        fees.fee
    };

    let total_fee = (adjusted_fee_basis_points as u128)
        .checked_mul(amount as u128)
        .ok_or(ErrorCode::NumericalOverflow)?
        .checked_div(10000)
        .ok_or(ErrorCode::NumericalOverflow)? as u64;

    let seller_amount = match fees.fee_payer {
        PaymentFeePayer::Buyer => amount,
        PaymentFeePayer::Seller => amount.checked_sub(total_fee).ok_or(ErrorCode::NumericalOverflow)?,
    };

    Ok((total_fee, seller_amount))
}