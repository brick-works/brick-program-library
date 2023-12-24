use anchor_lang::solana_program::pubkey::Pubkey;
use crate::MARKETPLACE_PROGRAM;


fn get_address(seeds: &[&[u8]], program_id: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(seeds, program_id).0
}

pub fn get_access_address(authority: &Pubkey, marketplace: &Pubkey) -> Pubkey{
    get_address(
        &[
            b"access".as_ref(),
            authority.as_ref(),
            marketplace.as_ref(),
        ],
        &MARKETPLACE_PROGRAM
    )
}

pub fn get_marketplace_address(signer: &Pubkey) -> Pubkey {
    get_address(
        &[
            b"marketplace".as_ref(),
            signer.as_ref(),
        ],
        &MARKETPLACE_PROGRAM
    )
}

pub fn get_access_mint_address(marketplace: &Pubkey) -> Pubkey {
    get_address(
        &[
            b"access_mint".as_ref(),
            marketplace.as_ref(),
        ],
        &MARKETPLACE_PROGRAM
    )
}

pub fn get_bounty_address(marketplace: &Pubkey, reward: &Pubkey) -> Pubkey {
    get_address(
        &[
            b"bounty_vault".as_ref(),
            marketplace.as_ref(),
            reward.as_ref(),
        ],
        &MARKETPLACE_PROGRAM
    )
}

pub fn get_product_address(product_id: &[u8; 16]) -> Pubkey {
    get_address(
        &[
            b"product".as_ref(),
            product_id.as_ref(),
        ],
        &MARKETPLACE_PROGRAM
    )
}