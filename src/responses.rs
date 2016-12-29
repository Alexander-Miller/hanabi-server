use game_state::GameState;
use self::ResponseType::*;

#[derive(Debug)]
pub enum ResponseType {
    ErrorResponseType,
    ConnectionResponseType,
    DiscardCardResponseType,
    PlayCardResponseType,
    HintColorResposeType,
    HintNumberResposeType,
    GameOverResponseType,
    GameStartResponseType,
}

#[derive(RustcEncodable)]
pub struct ErrorResponse {
    msg_type:    ResponseType,
    explanation: &'static str,
    err_details: Option<String>,
}

impl ErrorResponse {
    pub fn new(explanation: &'static str, err_details: Option<&str>) -> Self {
        ErrorResponse {
            msg_type:    ErrorResponseType,
            explanation: explanation,
            err_details: match err_details {
                Some(details) => Some(details.to_owned()),
                None          => None,
            }
        }
    }
}

#[derive(RustcEncodable)]
pub struct ConnectionResponse {
    msg_type: ResponseType,
    name:     String
}

impl ConnectionResponse {
    pub fn new<S>(name: S) -> Self
        where S: Into<String> {
        ConnectionResponse {
            msg_type: ConnectionResponseType,
            name:     name.into()
        }
    }
}

#[derive(RustcEncodable)]
pub struct DiscardCardResponse<'s> {
    msg_type:    ResponseType,
    next_player: &'s str,
    game_state:  &'s GameState,
}

impl<'s> DiscardCardResponse<'s> {
    pub fn new(next_player: &'s str, game_state: &'s GameState) -> Self {
        DiscardCardResponse {
            msg_type:    DiscardCardResponseType,
            next_player: next_player,
            game_state:  game_state,
        }
    }
}

#[derive(RustcEncodable)]
pub struct PlayCardResponse<'s> {
    msg_type:    ResponseType,
    next_player: &'s str,
    game_state:  &'s GameState,
}

impl<'s> PlayCardResponse<'s> {
    pub fn new(next_player: &'s str, game_state: &'s GameState) -> Self {
        PlayCardResponse {
            msg_type:    PlayCardResponseType,
            next_player: next_player,
            game_state:  game_state,
        }
    }
}

#[derive(RustcEncodable)]
pub struct HintColorResponse<'s> {
    msg_type:    ResponseType,
    next_player: &'s str,
    game_state:  &'s GameState,
}

impl<'s> HintColorResponse<'s> {
    pub fn new(next_player: &'s str, game_state: &'s GameState) -> Self {
        HintColorResponse {
            msg_type:    HintColorResposeType,
            next_player: next_player,
            game_state:  game_state,
        }
    }
}

#[derive(RustcEncodable)]
pub struct HintNumberResponse<'s> {
    msg_type:    ResponseType,
    next_player: &'s str,
    game_state:  &'s GameState,
}

impl<'s> HintNumberResponse<'s> {
    pub fn new(next_player: &'s str, game_state: &'s GameState) -> Self {
        HintNumberResponse {
            msg_type:    HintNumberResposeType,
            next_player: next_player,
            game_state:  game_state,
        }
    }
}

#[derive(RustcEncodable)]
pub struct GameOverResponse {
    msg_type: ResponseType,
    score:    usize
}

impl GameOverResponse {
    pub fn new(score: usize) -> Self {
        GameOverResponse {
            msg_type: GameOverResponseType,
            score:    score
        }
    }
}

#[derive(RustcEncodable)]
pub struct GameStartResponse {
    msg_type: ResponseType
}

impl GameStartResponse {
    pub fn new() -> Self {
        GameStartResponse {
            msg_type: GameStartResponseType
        }
    }
}

pub mod error_messages {
    pub const MSG_TO_TXT_ERROR:              &'static str = "The received message could not be read as a String.";
    pub const UNABLE_TO_GET_MSG_TYPE:        &'static str = "The type of the message could not be read.";
    pub const UNABLE_TO_DESERIALIZE_PAYLOAD: &'static str = "The payload of the received message could not be deserialized.";
    pub const CATASTROPHIC_FUCKUP:           &'static str = "Catastrophic Fuckup! The server's done goofed.";
    pub const PLAYER_ALREADY_EXISTS:         &'static str = "A Player with the chosen name already exists.";
    pub const ALREADY_CONNECTED:             &'static str = "The Player is already connected.";
    pub const GAME_ALREADY_STARTED:          &'static str = "Connection refused because the game has already started.";
    pub const NOT_YET_CONNECTED:             &'static str = "The Player is not yet connected.";
    pub const NO_CARDS:                      &'static str = "The deck has nor more cards.";
    pub const NO_HINT_TOKENS:                &'static str = "Hint token count is zero, a hint cannot be played.";
    pub const CARD_NOT_FOUND:                &'static str = "The given Card cannot be found on the Player's hand.";
}
