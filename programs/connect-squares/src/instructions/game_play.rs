
use anchor_lang::prelude::*;
use crate::{
    state::{
        game::*,
        Pot, 
        Metadata,
    },
    errors::GameError,
    utils::transfer_owned_sol,
};


pub fn game_play_handler(ctx: Context<GamePlay>, tile: Tile) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let player = &ctx.accounts.player;

    game.play(player.key(), &tile)?; //validates that player is a valid player

    if let GameState::Won{winner} = game.get_state() {
        require_keys_eq!(winner, player.key(), GameError::PlayerWinnerMismatch);

        //transfer pot to winner
        let pot = &mut ctx.accounts.pot.to_account_info();
        let winnings = game.get_wager() as u64 * game.get_player_count() as u64;
        transfer_owned_sol(pot,
            &mut player.to_account_info(),
            winnings)?;

        transfer_owned_sol(pot, &mut ctx.accounts.metadata.to_account_info(), pot.lamports())
    } else {
        Ok(())
    }
}



#[derive(Accounts)]
pub struct GamePlay<'info> {
    #[account(
        mut,
        seeds = [b"game", game.get_creator().as_ref(), &game.get_nonce().to_be_bytes()],
        bump = game.get_bump(),
    )]
    pub game: Box<Account<'info, Game>>,

    #[account(
        mut,
        seeds = [b"pot", game.key().as_ref()],
        bump=pot.bump,
    )]
    pub pot: Account<'info, Pot>,

    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"metadata"],
        bump = metadata.get_bump(),
    )]
    pub metadata: Account<'info, Metadata>,

    pub system_program: Program<'info, System>,
}

