
use crate::state::game::*;
use anchor_lang::prelude::*;


pub fn game_play_handler(ctx: Context<GamePlay>, tile: Tile) -> Result<()> {
    ctx.accounts.game.play(ctx.accounts.player.key(), &tile)
}


#[derive(Accounts)]
pub struct GamePlay<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub player: Signer<'info>,
}

