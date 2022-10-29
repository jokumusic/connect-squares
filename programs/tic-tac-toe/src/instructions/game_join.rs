use anchor_lang::prelude::*;
use crate::state::{Game, Pot};

pub fn game_join_handler(ctx: Context<GameJoin>) -> Result<()> {
    ctx.accounts
        .game
        .join(ctx.accounts.player.key())
}

#[derive(Accounts)]
pub struct GameJoin<'info> {
    #[account(
        mut,
        seeds = [b"game", game.get_creator().as_ref()],
        bump = game.get_bump()
    )]
    pub game: Account<'info, Game>,

    #[account(
        mut,
        seeds = [b"pot", game.key().as_ref()],
        bump = pot.bump,
    )]
    pub pot: Account<'info, Pot>,

    #[account(mut)]
    pub player: Signer<'info>,
}