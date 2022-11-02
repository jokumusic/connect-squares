use anchor_lang::prelude::*;
use crate::errors::GameError;

const PLAYER_TURN_MAX_SLOTS: u8 = 240;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Copy)]
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
    bump: u8, //1;
    creator: Pubkey, //32;
    nonce: u32, //4;
    state: GameState, //1+32
    rows: u8, //1;
    cols: u8, //1;
    min_players: u8, //1;
    max_players: u8, //1;
    moves: u8, //1;
    wager: u32, //4;
    pot: Pubkey, //32;
    init_timestamp: i64, //8;
    last_move_slot: u64, //8;
    joined_players: u8, //1;
    current_player_index: u8, //1;
    board: Vec<Vec<Option<u8>>>, //dynamic;
    players: Vec<Pubkey>, //dynamic;
}

impl Game {
    pub const SIZE: usize = 1 + 32 + 4 + (1+32) + 1 + 1 + 1 + 1 + 1 + 4 + 32 + 8 + 8 + 1;

    pub fn init(&mut self, bump: u8, creator: Pubkey, nonce: u32, pot:Pubkey, rows: u8, cols: u8, min_players: u8, max_players: u8, wager: u32) -> Result<()> {
        require!(rows > 2, GameError::RowsMustBeGreaterThanTwo);
        require!(cols > 2, GameError::ColumnsMustBeGreaterThanTwo);
        require!(min_players > 1, GameError::MinimumPlayersMustBeGreaterThanOne);
        require!(max_players > 1, GameError::MaximumPlayersMustBeGreaterThanOne);
        require!(max_players >= min_players, GameError::MaximumPlayersMustBeGreaterThanOrEqualToMiniumPlayers);

        self.bump = bump;
        self.creator = creator;
        self.nonce = nonce;
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
            self.last_move_slot =  Clock::get()?.slot;
        }

        Ok(())
    }

    pub fn play(&mut self, player: Pubkey, tile: &Tile) -> Result<()> {
        require!(self.is_active(), GameError::GameAlreadyOver);
        require!(tile.row < self.rows, GameError::TileOutOfBounds);
        require!(tile.column < self.cols, GameError::TileOutOfBounds);

        let calculated_player_index = self.calculate_current_player_index();
        let calculated_player_pubkey = self.players[calculated_player_index as usize];
        let slot = Clock::get()?.slot;
        
        /*
        let slot = Clock::get()?.slot;
        let slot_expired = slot - self.last_move_slot > u64::from(PLAYER_TURN_MAX_SLOTS);
        if slot_expired {
            self.current_player_index += 1;
            if self.current_player_index >= self.joined_players {
                self.current_player_index = 0;
            }
        } 
        */
        require_keys_eq!(calculated_player_pubkey, player, GameError::NotPlayersTurn);
        
        let cell_value = self.board[tile.row as usize][tile.column as usize];
        match cell_value {
            Some(_) => return Err(GameError::TileAlreadySet.into()),
            None => {
                self.board[tile.row as usize][tile.column as usize] = Some(calculated_player_index as u8);
                self.last_move_slot = slot;
                self.moves += 1;           
                self.current_player_index = calculated_player_index as u8;     
            }
        }       
        

        if self.row_all_equal(tile.row as usize) || self.col_all_equal(tile.column as usize) || self.positive_slope_all_equal() || self.negative_slope_all_equal() {
            self.state = GameState::Won {
                winner: player,
            };            
        }
        else if self.moves == self.cols * self.rows {
            //self.state = GameState::Tie;
            self.board = vec![vec![None; self.cols as usize]; self.rows as usize]; //reset board. This is a deathmatch - ties don't exist.
        }

        if GameState::Active == self.state {
            self.current_player_index = (calculated_player_index as u8) + 1;
            if self.current_player_index >= self.joined_players {
                self.current_player_index = 0;
            }
        }

        Ok(())
    }

    pub fn get_bump(&self) -> u8 {
        self.bump
    }

    pub fn get_creator(&self)-> Pubkey {
        self.creator
    }

    pub fn get_nonce(&self)-> u32 {
        self.nonce
    }

    pub fn get_state(&self) -> GameState {
        self.state
    }

    pub fn get_wager(&self) -> u32 {
        self.wager
    }

    pub fn is_active(&self) -> bool {
        self.state == GameState::Active
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

    fn positive_slope_all_equal(&self) -> bool {
        if self.rows != self.cols {
            return false;
        }

        let mut current_row = self.board.len() - 1;
        
        if self.board[current_row][0].is_some() {
            let mut rejected = false;

            for col in 1..self.board[0].len() {
                current_row -= 1;
                let current_cell_player = self.board[current_row][col];
                if current_cell_player.is_none() || current_cell_player != self.board[current_row+1][col-1] {
                    rejected = true;
                    break;
                }
            }

            if !rejected {
                //msg!("positive slope win!");
                return true;
            }
        }

        false
    }

    fn negative_slope_all_equal(&self) -> bool {
        if self.rows != self.cols {
            return false;
        }
        
        let mut rejected = false;
        
        for i in 1..self.board.len() {
            //msg!("i={}, prev: {:?}, current: {:?}", i, self.board[i-1][i-1], self.board[i][i]);
            let current_cell_player = self.board[i][i];
            if current_cell_player.is_none() || current_cell_player != self.board[i-1][i-1] {
                rejected = true;
                break;
            }
        }

        if !rejected {
            //msg!("negative slope win!");
            return true;
        }        

        false
    }

    fn calculate_current_player_index(&self)-> usize {
        let slot = Clock::get().unwrap().slot;
        let slot_diff = slot - self.last_move_slot;
        let turns_passed =  slot_diff / (PLAYER_TURN_MAX_SLOTS as u64);
        let mut player_index = self.current_player_index;
        let adder;
        if turns_passed >= self.joined_players as u64 {
            adder = (turns_passed % (self.joined_players as u64)) as u8;
        } else{
            adder = turns_passed as u8;
        }

        player_index += adder;
      
        if player_index >= self.joined_players {
            player_index = 0;
        }

        //msg!("last_move_slot: {}, slot: {}, slot_diff: {}, turns_passed: {}, adder: {}, player_index: {}", 
        //    self.last_move_slot, slot, slot_diff, turns_passed, adder, player_index);
        player_index as usize
    }

}