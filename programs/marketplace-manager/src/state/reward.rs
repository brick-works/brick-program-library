use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, TransferChecked};
use crate::error::ErrorCode;

#[account]
pub struct Reward {
    pub authority: Pubkey,
    pub bump: u8
}

impl Reward {
    pub const SIZE: usize = 8 + 32;

    pub fn calculate_bonus(
        rewards_percentage: u16,
        product_price: u64,
    ) -> Result<u64> {
        let bonus = (rewards_percentage as u128)
            .checked_mul(product_price as u128)
            .ok_or(ErrorCode::NumericalOverflow)?
            .checked_div(10000)
            .ok_or(ErrorCode::NumericalOverflow)? as u64;
        Ok(bonus)
    }
    
    pub fn validate_vault(vault_owner: Pubkey, proof: Pubkey) -> bool {
        vault_owner == proof
    }
    
    pub fn transfer_bonus<'info>(
        from: AccountInfo<'info>,
        mint: AccountInfo<'info>,
        to: AccountInfo<'info>,
        marketplace: AccountInfo<'info>,
        token_program: AccountInfo<'info>,
        bonus: u64,
        decimals: u8,
        marketplace_seeds: &[&[u8]; 3],
    ) -> Result<()> {
        transfer_checked(
            CpiContext::new_with_signer(
                token_program,
                TransferChecked {
                    from,
                    mint,
                    to,
                    authority: marketplace,
                },
                &[&marketplace_seeds[..]],
            ),
            bonus,
            decimals,
        )?;
    
        Ok(())
    }
    
}