use anchor_lang::prelude::*;
use crate::errors::ProgramError;

#[account]
pub struct Metadata {
    bump: u8, //1;
    initialized: bool, //1;
    authority: Pubkey, //32;
}

impl Metadata {
    pub const SIZE: usize = 1 + 1 + 32;

    pub fn init(&mut self, bump: u8, authority: Pubkey) -> Result<()> {
        require!(!self.initialized, ProgramError::AlreadyInitialized);
        
        self.bump = bump;
        self.initialized = true;
        self.authority = authority; 

        Ok(())
    }

    pub fn set_authority(&mut self, old_authority: Pubkey, new_authority: Pubkey) -> Result<()> {
        require_keys_eq!(self.authority, old_authority, ProgramError::Unauthorized);

        self.authority = new_authority;
        
        Ok(())
    }

    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    pub fn get_authority(&self) -> Pubkey {
        self.authority
    }
}