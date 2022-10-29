use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;

pub mod state;
use crate::state::Tile;

pub mod errors;


// this key needs to be changed to whatever public key is returned by "anchor keys list"
declare_id!("H5k95qzHVCoKJSDCE5WLJ9kcmfSWn89sw4gWkjGY76DB");

#[program]
pub mod tic_tac_toe {
    use super::*;

    pub fn game_init(ctx: Context<GameInit>, rows: u8, cols: u8, min_players: u8, max_players: u8, wager: u32) -> Result<()> {
        instructions::game_init_handler(ctx, rows, cols, min_players, max_players, wager)
    }

    pub fn game_join(ctx: Context<GameJoin>) -> Result<()> {
        instructions::game_join_handler(ctx)
    }

    pub fn game_play(ctx: Context<GamePlay>, tile: Tile) -> Result<()> {
        instructions::game_play_handler(ctx, tile)
    }
}
