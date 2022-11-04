use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;

pub mod state;
use crate::state::Tile;

pub mod errors;
pub mod utils;


// this key needs to be changed to whatever public key is returned by "anchor keys list"
declare_id!("Fd8xjFh4Nk2RCR9zrvxJncDBfZo9ypqwFoiGnnX7YVC8");

#[program]
pub mod tic_tac_toe {
    use super::*;

    pub fn game_init(ctx: Context<GameInit>, nonce: u32, rows: u8, cols: u8, connect: u8, min_players: u8, max_players: u8, wager: u32) -> Result<()> {
        instructions::game_init_handler(ctx, nonce, rows, cols, connect, min_players, max_players, wager)
    }

    pub fn game_cancel(ctx: Context<GameCancel>) -> Result<()> {
        instructions::game_cancel_handler(ctx)
    }

    pub fn game_join(ctx: Context<GameJoin>) -> Result<()> {
        instructions::game_join_handler(ctx)
    }

    pub fn game_play(ctx: Context<GamePlay>, tile: Tile) -> Result<()> {
        instructions::game_play_handler(ctx, tile)
    }
}
