use anchor_lang::prelude::*;
use crate::state::{game::*, Pot};



pub fn game_init_handler(ctx: Context<GameInit>, nonce: u32, rows: u8, cols: u8, min_players: u8, max_players: u8, wager: u32) -> Result<()> {
    let pot = &mut ctx.accounts.pot;
    let pot_bump = *ctx.bumps.get("pot").unwrap();
    pot.init(pot_bump, ctx.accounts.game.key())?;

    let bump = *ctx.bumps.get("game").unwrap();
    let creator_key = ctx.accounts.creator.key();
    ctx.accounts.game.init(bump, creator_key, nonce, pot.key(), rows, cols, min_players, max_players, wager)
}


#[derive(Accounts)]
#[instruction(nonce: u32, rows: u8, cols: u8, min_players: u8, max_players: u8, wager: u32)]
pub struct GameInit<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init, 
        payer = creator,
        space = 8 + Game::SIZE + usize::from(max_players * 32) + usize::from(4 * rows * (2 * cols)),
        seeds = [b"game", creator.key().as_ref(), &nonce.to_be_bytes()],
        bump,
    )]
    pub game: Box<Account<'info, Game>>,
 
    #[account(
        init,
        payer = creator,
        space = 8 + Pot::SIZE,
        seeds = [b"pot", game.key().as_ref()],
        bump,
    )]
    pub pot: Account<'info, Pot>,
    pub system_program: Program<'info, System>,
}
