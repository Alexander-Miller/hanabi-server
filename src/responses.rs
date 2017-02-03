use game_state::GameState;
use cards::{Color, Number, Card};
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
pub struct ConnectionResponse<'s> {
    msg_type: ResponseType,
    names:    Vec<&'s str>,
}

impl<'s> ConnectionResponse<'s> {
    pub fn new(names: Vec<&'s str>) -> Self {
        ConnectionResponse {
            msg_type: ConnectionResponseType,
            names:    names,
        }
    }
}

#[derive(RustcEncodable)]
pub struct DiscardCardResponse<'s> {
    msg_type:          ResponseType,
    discarding_player: &'s str,
    discarded_card:    &'s Card,
    drawn_card:        Option<&'s Card>,
    game_state:        &'s GameState,
}

impl<'s> DiscardCardResponse<'s> {
    pub fn new(discarding_player: &'s str,
               discarded_card: &'s Card,
               drawn_card: Option<&'s Card>,
               game_state: &'s GameState)
               -> Self {
        DiscardCardResponse {
            msg_type:          DiscardCardResponseType,
            discarding_player: discarding_player,
            discarded_card:    discarded_card,
            drawn_card:        drawn_card,
            game_state:        game_state,
        }
    }
}

#[derive(RustcEncodable)]
pub struct PlayCardResponse<'s> {
    msg_type:       ResponseType,
    playing_player: &'s str,
    played_card:    &'s Card,
    drawn_card:     Option<&'s Card>,
    success:        bool,
    game_state:     &'s GameState,
}

impl<'s> PlayCardResponse<'s> {
    pub fn new(playing_player: &'s str,
               played_card: &'s Card,
               drawn_card: Option<&'s Card>,
               success: bool,
               game_state: &'s GameState)
               -> Self {
        PlayCardResponse {
            msg_type:       PlayCardResponseType,
            playing_player: playing_player,
            played_card:    played_card,
            drawn_card:     drawn_card,
            success:        success,
            game_state:     game_state,
        }
    }
}

#[derive(RustcEncodable)]
pub struct HintColorResponse<'s> {
    msg_type:       ResponseType,
    hinting_player: &'s str,
    target_player:  &'s str,
    hinted_color:   &'s Color,
    game_state:     &'s GameState,
}

impl<'s> HintColorResponse<'s> {
    pub fn new(hinting_player: &'s str,
               target_player: &'s str,
               hinted_color: &'s Color,
               game_state: &'s GameState)
               -> Self {
        HintColorResponse {
            msg_type:       HintColorResposeType,
            hinting_player: hinting_player,
            target_player:  target_player,
            hinted_color:   hinted_color,
            game_state:     game_state,
        }
    }
}

#[derive(RustcEncodable)]
pub struct HintNumberResponse<'s> {
    msg_type:       ResponseType,
    hinting_player: &'s str,
    target_player:  &'s str,
    hinted_number:  &'s Number,
    game_state:     &'s GameState,
}

impl<'s> HintNumberResponse<'s> {
    pub fn new(hinting_player: &'s str,
               target_player: &'s str,
               hinted_number: &'s Number,
               game_state: &'s GameState)
               -> Self {
        HintNumberResponse {
            msg_type:       HintNumberResposeType,
            hinting_player: hinting_player,
            target_player:  target_player,
            hinted_number:  hinted_number,
            game_state:     game_state,
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
pub struct GameStartResponse<'s> {
    msg_type: ResponseType,
    game_state:  &'s GameState,
}

impl<'s> GameStartResponse<'s> {
    pub fn new(game_state: &'s GameState) -> Self {
        GameStartResponse {
            msg_type:    GameStartResponseType,
            game_state:  game_state,
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
    pub const PLAYER_NOT_FOUND:              &'static str = "The given Player could not be found.";
    pub const GAME_IS_OVER:                  &'static str = "Tried to make a turn pass when no turns are left.";
}
