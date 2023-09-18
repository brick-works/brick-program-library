pub mod mint_builder;
pub mod handle_payment;
pub mod token_metadata;

pub use mint_builder::*;
pub use handle_payment::*;
pub use token_metadata::*;

use anchor_lang::{
    prelude::*,
    solana_program::{program_memory::sol_memcmp, pubkey::PUBKEY_BYTES},
};
use crate::{state::{MarketplaceBumps, RewardsConfig}, error::ErrorCode};
use spl_token::native_mint::ID as NativeMint;

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
}

pub fn get_bounty_bump(address: Pubkey, bumps: MarketplaceBumps, bounty_vaults: Vec<Pubkey>) -> u8 {
    bounty_vaults.iter().position(|&r| r == address)
        .map(|index| bumps.vault_bumps[index])
        .unwrap_or(0)
}

/// Checks if marketplace reward system is active, is active when:
/// If reward_mint == null_mint && rewardsEnabled == false -> NO REWARDS
/// If reward_mint == null_mint && rewardsEnabled == true -> REWARDS (regardless of the reward_mint)
/// If reward_mint == mint && rewardsEnabled == true -> REWARDS only with specific reward_mint
/// CANT BE NATIVE MINT (ie SOL), a PDA from my program cant transfer SOL because is not owned by SystemProgram
pub fn is_rewards_active(reward_config: RewardsConfig, payment_mint: Pubkey, program_id: Pubkey) -> bool {
    let null_seeds = &[b"null".as_ref()];
    let account_address = Pubkey::find_program_address(null_seeds, &program_id);
    
    reward_config.rewards_enabled && !cmp_pubkeys(&payment_mint, &NativeMint)
        && (cmp_pubkeys(&payment_mint, &reward_config.reward_mint) || cmp_pubkeys(&reward_config.reward_mint, &account_address.0))
}

pub fn assert_ata(
    account: &AccountInfo,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<()> {
    assert_derivation(
        &anchor_spl::associated_token::ID,
        &account.to_account_info(),
        &[
            owner.as_ref(),
            anchor_spl::token::ID.as_ref(),
            mint.as_ref(),
        ],
    )
}

pub fn assert_derivation(
    program_id: &Pubkey,
    account: &AccountInfo,
    path: &[&[u8]],
) -> Result<()> {
    let key = Pubkey::find_program_address(path, program_id);
    if !cmp_pubkeys(&key.0, account.key) {
        return Err(ErrorCode::IncorrectSeeds.into());
    }
    Ok(())
}

pub fn assert_authority(account_property: &Pubkey, auth: &Pubkey) -> Result<()> {
    if !cmp_pubkeys(account_property, auth) {
        Err(ErrorCode::IncorrectAuthority.into())
    } else {
        Ok(())
    }
}