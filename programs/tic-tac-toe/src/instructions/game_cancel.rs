use anchor_lang::prelude::*;
use crate::{
    state::{
        game::*,
        Pot,
    },
    errors::GameError
};


pub fn game_cancel_handler(ctx: Context<GameCancel>) -> Result<()> {
    let game = &mut ctx.accounts.game;
    let player = &ctx.accounts.player;

    game.cancel(player.key())
}

#[derive(Accounts)]
pub struct GameCancel<'info> {
    #[account(
        mut,
        seeds = [b"game", game.get_creator().as_ref(), &game.get_nonce().to_be_bytes()],
        bump = game.get_bump(),
        close = player,
        constraint = player.key() == game.get_creator() @ GameError::NotAuthorized,
    )]
    pub game: Box<Account<'info, Game>>,

    #[account(
        mut,
        seeds = [b"pot", game.key().as_ref()],
        bump=pot.bump,
        close = player,
    )]
    pub pot: Account<'info, Pot>,

    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}