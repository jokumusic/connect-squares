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
    #[msg("rows must be greater than 2")]
    RowsMustBeGreaterThanTwo,
    #[msg("colums must be greater than 2")]
    ColumnsMustBeGreaterThanTwo,
    #[msg("minimum players must be greater than 1")]
    MinimumPlayersMustBeGreaterThanOne,
    #[msg("maximum players must be greater than 1")]
    MaximumPlayersMustBeGreaterThanOne,
    #[msg("maximum players must be greater than or equal to minimum players")]
    MaximumPlayersMustBeGreaterThanOrEqualToMiniumPlayers,
    #[msg("failed to transfer funds")]
    FailedToTransferFunds,
    #[msg("too many players specified")]
    TooManyPlayersSpecified,
    #[msg("connect minimum not met")]
    ConnectMinimumNotMet,
    #[msg("connect cannot be greater than the number of rows")]
    ConnectIsGreaterThanNumberOfRows,
    #[msg("connect cannot be greater than the number of columns")]
    ConnectIsGreaterThanNumberOfColumns,
}
