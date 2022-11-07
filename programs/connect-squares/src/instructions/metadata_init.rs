use anchor_lang::prelude::*;
use crate::state::Metadata;

pub fn metadata_init_handler(ctx: Context<MetadataInit>) -> Result<()> {
    let bump = *ctx.bumps.get("metadata").unwrap();
    ctx.accounts.metadata.init(bump, ctx.accounts.authority.key())
}

#[derive(Accounts)]
pub struct MetadataInit<'info> {
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer=authority,
        space = 8 + Metadata::SIZE,
        seeds = [b"metadata"],
        bump
    )]
    pub metadata: Account<'info, Metadata>,
    pub system_program: Program<'info, System>,
}