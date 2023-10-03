use anchor_lang::prelude::*;

#[account]
pub struct Network {
    pub council_collection: Pubkey,
    pub service_collection: Pubkey,
    pub proposal_collection: Pubkey,
    pub network_mint: Pubkey,
    pub council_collection_bump: u8,
    pub service_collection_bump: u8,
    pub proposal_collection_bump: u8,
    pub mint_bump: u8,
    pub bump: u8,
}

pub const NETWORK_SIZE: usize = 8 + 32 + 32 + 32 + 32 + 1 + 1 + 1 + 1 + 1; 

#[account]
pub struct Proposal {
    pub id: [u8; 16],
    pub authority: Pubkey,
    pub vault: Pubkey,
    pub state: RequestState,
    pub vault_bump: u8,
    pub bump: u8,
    pub description: String,
}

pub const PROPOSAL_SIZE: usize = 8 + 16 + 32 + 32 + 8 + 1 + 1 + 132;

#[account]
pub struct Request {
    pub price: u64,
    pub payment_mint: u64,
    pub state: RequestState,
    pub bump: u8,
}

pub const REQUEST_SIZE: usize = 8 + 8 + 8 + 1;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum RequestState {
    /// SigningOff - The Proposal is being signed off by Signatories
    /// Proposal enters the state when first Signatory Sings and leaves it when last Signatory signs
    SigningOff,
    /// Taking votes
    Voting,
    /// Voting ended with success
    Succeeded,
    /// Completed
    Completed,
    /// Cancelled
    Cancelled,
    /// Defeated
    Defeated,
}
