use anchor_lang::prelude::*;
use crate::state::Metadata;

pub fn metadata_set_authority_handler(ctx: Context<MetadataSetAuthority>, new_authority: Pubkey) -> Result<()> {
    ctx.accounts.metadata.set_authority(ctx.accounts.authority.key(), new_authority)
}

#[derive(Accounts)]
pub struct MetadataSetAuthority<'info> {
    
    #[account()]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"metadata"],
        bump = metadata.get_bump(),
    )]
    pub metadata: Account<'info, Metadata>,
    pub system_program: Program<'info, System>,
}