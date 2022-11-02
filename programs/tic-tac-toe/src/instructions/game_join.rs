use anchor_lang::prelude::*;
use crate::state::{Game, Pot};

pub fn game_join_handler(ctx: Context<GameJoin>) -> Result<()> {
    //transfer wager to pot    
    let from = ctx.accounts.player.to_account_info();
    let to = ctx.accounts.pot.to_account_info();
    //transfer_sol(from, to, u64::from(wager))?;
    let ix = anchor_lang::solana_program::system_instruction::transfer(
       from.key,
       to.key,
       u64::from(ctx.accounts.game.get_wager()),
   );

    anchor_lang::solana_program::program::invoke(
       &ix,
       &[from, to]
    )?;

    ctx.accounts
        .game
        .join(ctx.accounts.player.key())
}

#[derive(Accounts)]
pub struct GameJoin<'info> {
    #[account(
        mut,
        seeds = [b"game", game.get_creator().as_ref(), &game.get_nonce().to_be_bytes()],
        bump = game.get_bump()
    )]
    pub game: Box<Account<'info, Game>>,

    #[account(
        mut,
        seeds = [b"pot", game.key().as_ref()],
        bump = pot.bump,
    )]
    pub pot: Account<'info, Pot>,

    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}