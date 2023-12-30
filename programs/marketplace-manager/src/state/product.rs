use {
    crate::error::ErrorCode,
    crate::state::*,
    anchor_lang::prelude::*,   
    anchor_spl::token::{transfer_checked, TransferChecked},
    spl_token_2022::cmp_pubkeys
};

/// This account works as an product administrator
#[account]
pub struct Product {
    /// Seller
    pub authority: Pubkey,
    pub marketplace: Pubkey,
    pub id: [u8; 16],
    /// Seller-defined product configurations.
    pub seller_config: SellerConfig,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SellerConfig {
    pub payment_mint: Pubkey,
    pub product_price: u64,
}

impl Product {
    pub const SIZE: usize = 8 + 32 + 32 + 16 + 32 + 8;

    pub fn initialize(
        &mut self, 
        authority: Pubkey, 
        marketplace: Pubkey,
        id: [u8; 16],
        seller_config: SellerConfig, 
    ) -> Result<()> {
        self.authority = authority;
        self.marketplace = marketplace;
        self.id = id;
        self.seller_config = seller_config;

        Ok(())
    }
    
    pub fn do_payment<'info>(
        signer: AccountInfo<'info>,
        buyer_vault: AccountInfo<'info>,
        seller_vault: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        amount: u64,
        decimals: u8,
    ) -> Result<()> {        
        transfer_checked(
            CpiContext::new(
                token_program,
                TransferChecked {
                    from: buyer_vault,
                    mint,
                    to: seller_vault,
                    authority: signer,
                },
            ),
            amount,
            decimals,
        )?;

        Ok(())
    }
    
    pub fn do_fee_payment<'info>(
        signer: AccountInfo<'info>,
        buyer_vault: AccountInfo<'info>,
        seller_vault: AccountInfo<'info>,
        marketplace_vault: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        fees_config: FeesConfig,
        payment_mint: Pubkey,
        amount: u64,
        decimals: u8,
    ) -> Result<()> {
        let (market_amount, seller_amount) =
            Product::calculate_transfer_distribution(fees_config, payment_mint, amount)?;
    
        transfer_checked(
            CpiContext::new(
                token_program.clone(),
                TransferChecked {
                    from: buyer_vault.clone(),
                    mint: mint.clone(),
                    to: marketplace_vault,
                    authority: signer.clone(),
                },
            ),
            market_amount,
            decimals,
        )?;

        transfer_checked(
            CpiContext::new(
                token_program,
                TransferChecked {
                    from: buyer_vault,
                    mint,
                    to: seller_vault,
                    authority: signer,
                },
            ),
            seller_amount,
            decimals,
        )?;
    
        Ok(())
    }    
    
    /// Calculates the distribution of the token amount, considering transaction fee and potential fee reduction.
    /// Adjusts the fee if the payment mint is the same as the discount mint.
    /// Also is considered fee_payer decided by the marketplace.
    fn calculate_transfer_distribution(
        fees: FeesConfig,
        payment_mint: Pubkey,
        amount: u64,
    ) -> std::result::Result<(u64, u64), ErrorCode> {
        let adjusted_fee_basis_points: u16 = match fees.discount_mint {
            Some(discount_mint) if cmp_pubkeys(&payment_mint, &discount_mint) => {
                fees.fee.saturating_sub(fees.fee_reduction.unwrap_or(0))
            }
            _ => fees.fee,
        };
    
        // Calculate total fee
        let total_fee = (adjusted_fee_basis_points as u128)
            .checked_mul(amount as u128)
            .ok_or(ErrorCode::NumericalOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::NumericalOverflow)? as u64;
    
        // Calculate seller amount based on fee payer
        let seller_amount = match fees.fee_payer {
            PaymentFeePayer::Buyer => amount,
            PaymentFeePayer::Seller => amount.checked_sub(total_fee).ok_or(ErrorCode::NumericalOverflow)?,
        };
    
        Ok((total_fee, seller_amount))
    }    
}