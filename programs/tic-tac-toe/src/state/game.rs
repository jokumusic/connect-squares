use anchor_lang::prelude::*;
use crate::errors::GameError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GameState {
    Waiting,
    Active,
    Tie,
    Won { winner: Pubkey },
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Tile {
    row: u8,
    column: u8,
}

#[account]
pub struct Game {
    pub bump: u8, //1;
    pub creator: Pubkey, //32;
    pub state: GameState, //1+32
    pub rows: u8, //1;
    pub cols: u8, //1;
    pub min_players: u8, //1;
    pub max_players: u8, //1;
    pub moves: u8, //1;
    pub wager: u32, //4;
    pub pot: Pubkey, //32;
    pub init_timestamp: i64, //8;
    pub last_move_slot: u64, //8;
    pub joined_players: u8, //1;
    pub current_player_index: u8, //1;
    pub board: Vec<Vec<Option<u8>>>, //dynamic;
    pub players: Vec<Pubkey>, //dynamic;
}

impl Game {
    pub const SIZE: usize = 1 + 32 + (1+32) + 1 + 1 + 1 + 1 + 1 + 4 + 32 + 8 + 8 + 1 + 1;

    pub fn init(&mut self, bump: u8, creator: Pubkey, pot:Pubkey, rows: u8, cols: u8, min_players: u8, max_players: u8, wager: u32) -> Result<()> {
        self.bump = bump;
        self.creator = creator;
        self.state = GameState::Waiting;
        self.rows = rows;
        self.cols = cols;
        self.min_players = min_players;
        self.max_players = max_players;
        self.moves = 0;
        self.wager = wager;
        self.pot = pot;
        self.last_move_slot = 0;
        self.joined_players = 1;
        self.current_player_index = 0;
        self.players = vec![Pubkey::default(); max_players as usize];
        self.board = vec![vec![None; cols as usize]; rows as usize];
        self.init_timestamp = Clock::get()?.unix_timestamp;

        self.players[0] = creator;

        Ok(())
    }

    pub fn join(&mut self, player: Pubkey) -> Result<()> {
        require!(self.state == GameState::Waiting, GameError::NotAcceptingPlayers);
        
        self.players[self.joined_players as usize] = player;
        self.joined_players += 1;

        if self.joined_players == self.min_players {
            self.state = GameState::Active;
        }

        Ok(())
    }

    pub fn play(&mut self, player: Pubkey, tile: &Tile) -> Result<()> {
        require!(self.is_active(), GameError::GameAlreadyOver);
        require_keys_eq!(self.current_player(), player, GameError::NotPlayersTurn);
        require!(tile.row < self.rows, GameError::TileOutOfBounds);
        require!(tile.column < self.cols, GameError::TileOutOfBounds);

        let current_player_index = self.current_player_index;
        
        let cell_value = self.board[tile.row as usize][tile.column as usize];
        match cell_value {
            Some(_) => return Err(GameError::TileAlreadySet.into()),
            None => {
                self.board[tile.row as usize][tile.column as usize] = Some(current_player_index);
                self.moves += 1;
            }
        }
        

        if self.row_all_equal(tile.row as usize) || self.col_all_equal(tile.column as usize) {
            self.state = GameState::Won {
                winner: self.current_player(),
            };
        }
        else if self.moves == self.cols * self.rows {
            self.state = GameState::Tie;
        }

        if GameState::Active == self.state {
            self.current_player_index += 1;
            if self.current_player_index >= self.joined_players {
                self.current_player_index = 0;
            }
        }

        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.state == GameState::Active
    }

    pub fn current_player(&self) -> Pubkey {
        self.players[self.current_player_index as usize]
    }

  


    fn row_all_equal(&self, row: usize) -> bool {
        if let Some(first) = self.board[row][0] {
            return self.board[row].iter().all(|&i| i == Some(first));
        }

        false
    }

    fn col_all_equal(&self, col: usize) -> bool {
        let first = self.board[0][col];
        if first.is_none() {
            return false;
        }

        for row in 1..self.board.len() {
            if self.board[row][col] != first {
                return false;
            }
        }        

        true
    }

}