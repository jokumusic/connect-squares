use anchor_lang::error_code;

#[error_code]
pub enum GameError {
    #[msg("specified tile is out of bounds")]
    TileOutOfBounds,
    #[msg("specified tile is occupied")]
    TileAlreadySet,
    #[msg("game has already ended")]
    GameAlreadyOver,
    #[msg("it is not your player's turn")]
    NotPlayersTurn,
    #[msg("game has already started")]
    GameAlreadyStarted,
    #[msg("game is not accepting new players")]
    NotAcceptingPlayers,
    #[msg("debiting the game pot has caused a numerical overflow")]
    PayoutDebitNumericalOverflow,
    #[msg("crediting the winner account has caused a numerical overflow")]
    PayoutCreditNumericalOverflow,
    #[msg("player and winner don't match")]
    PlayerWinnerMismatch,
}
