use anchor_lang::error_code;

#[error_code]
pub enum GameError {
    TileOutOfBounds,
    TileAlreadySet,
    GameAlreadyOver,
    NotPlayersTurn,
    GameAlreadyStarted,
    NotAcceptingPlayers,
}
