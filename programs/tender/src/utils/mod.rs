pub mod mint_builder;
pub mod token_metadata;

pub use mint_builder::*;
pub use token_metadata::*;

use anchor_lang::{
    prelude::*,
    solana_program::{program_memory::sol_memcmp, pubkey::PUBKEY_BYTES},
};
use crate::error::ErrorCode;

pub fn cmp_pubkeys(a: &Pubkey, b: &Pubkey) -> bool {
    sol_memcmp(a.as_ref(), b.as_ref(), PUBKEY_BYTES) == 0
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
