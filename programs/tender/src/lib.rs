pub mod state;
pub mod error;
mod instructions;
use {
    anchor_lang::prelude::*,
    instructions::*,
};

declare_id!("E6E7kfSE21wnKrpvtEQsCj3XFnZyeXu6UjoLcjogqbLQ");

#[program]
pub mod tender {
    use super::*;

    pub fn accept_request(ctx: Context<AcceptRequest>) -> Result<()> {
        accept_request::handler(ctx)
    }

    pub fn deny_request(ctx: Context<DenyRequest>) -> Result<()> {
        deny_request::handler(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        deposit::handler(ctx, amount)
    }

    pub fn do_request(ctx: Context<DoRequest>, price: u64) -> Result<()> {
        do_request::handler(ctx, price)
    }

    pub fn init_council_member(ctx: Context<InitCouncilMember>) -> Result<()> {
        init_council_member::handler(ctx)
    }

    pub fn init_network(ctx: Context<InitNetwork>, proposal_collection_uri: String) -> Result<()> {
        init_network::handler(ctx, proposal_collection_uri)
    }

    pub fn init_proposal(ctx: Context<InitProposal>, params: InitProposalParams) -> Result<()> {
        init_proposal::handler(ctx, params)
    }

    pub fn init_roles(ctx: Context<InitRoles>, params: InitRolesParams) -> Result<()> {
        init_roles::handler(ctx, params)
    }

    pub fn init_service_provider(ctx: Context<InitServiceProvider>) -> Result<()> {
        init_service_provider::handler(ctx)
    }
}
