use anchor_lang::prelude::*;

#[account]
pub struct Pot {
    pub bump: u8, //1;
    pub game: Pubkey, //32;
}

impl Pot {
    pub const SIZE: usize = 1 + 32;

    pub fn init(&mut self, bump: u8, game: Pubkey) -> Result<()> {
        self.bump = bump;
        self.game = game;

        Ok(())
    }
}
