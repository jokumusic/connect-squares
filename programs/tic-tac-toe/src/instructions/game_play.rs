
use anchor_lang::prelude::*;
use crate::{
    state::{
        game::*,
        Pot,
    },
    errors::GameError,
    utils::transfer_sol,
};


pub fn game_play_handler(ctx: Context<GamePlay>, tile: Tile) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let player = &ctx.accounts.player;

    game.play(player.key(), &tile)?; //validates that player is a valid player

    if let GameState::Won{winner} = game.get_state() {
        require_keys_eq!(winner, player.key(), GameError::PlayerWinnerMismatch);

        let from = &mut ctx.accounts.pot.to_account_info();
        let to = &mut player.to_account_info();
        let amount = from.lamports();
        transfer_sol(from, to, amount)?;
    }

    Ok(())
}



#[derive(Accounts)]
pub struct GamePlay<'info> {
    #[account(
        mut,
        seeds = [b"game", game.get_creator().as_ref(), &game.get_nonce().to_be_bytes()],
        bump = game.get_bump(),
    )]
    pub game: Account<'info, Game>,

    #[account(
        mut,
        seeds = [b"pot", game.key().as_ref()],
        bump=pot.bump,
    )]
    pub pot: Account<'info, Pot>,

    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

