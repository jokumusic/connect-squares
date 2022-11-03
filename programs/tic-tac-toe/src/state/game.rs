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
    connect: u8, //1;
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
    pub const SIZE: usize = 1 + 32 + 4 + (1+32) + 1 + 1 + 1 + 1 + 1 + 1 + 4 + 32 + 8 + 8 + 1;

    pub fn init(&mut self, bump: u8, creator: Pubkey, nonce: u32, pot:Pubkey, rows: u8, cols: u8, connect: u8, min_players: u8, max_players: u8, wager: u32) -> Result<()> {
        require!(rows > 2, GameError::RowsMustBeGreaterThanTwo);
        require!(cols > 2, GameError::ColumnsMustBeGreaterThanTwo);
        require!(min_players > 1, GameError::MinimumPlayersMustBeGreaterThanOne);
        require!(max_players > 1, GameError::MaximumPlayersMustBeGreaterThanOne);
        //only allow 2 players for now. More than two players allows collusion/cheating
        require!(min_players < 3, GameError::TooManyPlayersSpecified);
        require!(max_players < 3, GameError::TooManyPlayersSpecified);
        require!(max_players >= min_players, GameError::MaximumPlayersMustBeGreaterThanOrEqualToMiniumPlayers);
        require!(connect > 2, GameError::ConnectMinimumNotMet);

        self.bump = bump;
        self.creator = creator;
        self.nonce = nonce;
        self.state = GameState::Waiting;
        self.rows = rows;
        self.cols = cols;
        self.connect = connect;
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
            self.shuffle_players()?;
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
        

        if self.move_has_won(tile.row, tile.column) {
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

    fn shuffle_players(&mut self) -> Result<()> {
        let player_count = self.players.len() as u64;
        let clock = Clock::get()?;
        let seed_a = clock.unix_timestamp as u64; 
        let seed_b = clock.slot;

        for i in 1..player_count {   
            let a = ((seed_a / i) % player_count) as usize;
            let b = ((seed_b / i) % player_count) as usize;
            //msg!("seed_a={},seed_b={}, a={}, b={}", seed_a, seed_b, a, b);
            //add some self.players.reverse()? in here to make it more of a shuffle for larger player counts?
            if a != b {
                self.players.swap(a, b);                
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

    pub fn get_player_count(&self) -> u8 {
        self.joined_players
    }

    fn move_has_won(&self, row: u8, col: u8) -> bool {
        let row = row as i8;
        let col = col as i8;
        let adjacent_required = self.connect - 1;

        if self.adjacent_cell_count(row,col,0,1) + self.adjacent_cell_count(row,col,0,-1) >= adjacent_required //horizontal
        || self.adjacent_cell_count(row,col,1,0) + self.adjacent_cell_count(row,col,-1,0) >= adjacent_required //vertical
        || self.adjacent_cell_count(row,col,-1,1) + self.adjacent_cell_count(row,col,1,-1) >= adjacent_required //positive slope
        || self.adjacent_cell_count(row,col,1,1)+ self.adjacent_cell_count(row,col,-1,-1) >= adjacent_required //negative slope
        {
            return true
        }
        
        false
    }

    fn adjacent_cell_count(&self, row: i8,col:i8, row_increment: i8, col_increment: i8) -> u8 {
        if self.cell_value(row,col) == self.cell_value(row+row_increment, col+col_increment) {
            return 1 + self.adjacent_cell_count(row+row_increment, col+col_increment, row_increment, col_increment);
        }

        0
    }

    fn cell_value(&self, row: i8, col: i8) -> Option<u8> {
        let row = row as usize;
        let col = col as usize;
        if row >= self.board.len() || col >= self.board[0].len() {
            return None;
        }

        self.board[row][col]
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