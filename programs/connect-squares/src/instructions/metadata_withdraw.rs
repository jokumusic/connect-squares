use anchor_lang::prelude::*;
use crate::{
    state::Metadata,
    utils::transfer_owned_sol,
    errors::ProgramError
};

pub fn metadata_withdraw_handler(ctx: Context<MetadataWithdraw>, amount: u64) -> Result<()> {
    let metadata = &ctx.accounts.metadata;    
    let authority_account_info = &mut ctx.accounts.authority.to_account_info();
    
    require_keys_eq!(metadata.get_authority(), authority_account_info.key(), ProgramError::Unauthorized);

    let metadata_account_info = &mut metadata.to_account_info();
    require_gt!(metadata_account_info.lamports() - amount, 100000, ProgramError::InsufficientFunds); //must have enough funds remaining for rent
    
    transfer_owned_sol(metadata_account_info, authority_account_info, amount)
}

#[derive(Accounts)]
pub struct MetadataWithdraw<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"metadata"],
        bump = metadata.get_bump(),
    )]
    pub metadata: Account<'info, Metadata>,
    pub system_program: Program<'info, System>,
}