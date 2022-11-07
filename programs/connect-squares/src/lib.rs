use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;

pub mod state;
use crate::state::Tile;

pub mod errors;
pub mod utils;


declare_id!("ZG3VZPMEpziUq1RvcMJUbHr2dfWjHhWRjGHbgdg1LTR");

#[program]
pub mod connect_squares {
    use super::*;

    pub fn metadata_init(ctx: Context<MetadataInit>) -> Result<()> {
        instructions::metadata_init_handler(ctx)
    }

    pub fn metdata_set_authority(ctx: Context<MetadataSetAuthority>, new_authority: Pubkey) -> Result<()> {
        instructions::metadata_set_authority_handler(ctx, new_authority)
    }

    pub fn metadata_withdraw(ctx: Context<MetadataWithdraw>, amount: u64) -> Result<()> {
        instructions::metadata_withdraw_handler(ctx, amount)
    }

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
