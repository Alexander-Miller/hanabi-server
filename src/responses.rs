use rustc_serialize::{json, Encodable};
use game_state::GameState;

#[derive(Debug)]
pub enum ResponseType {
    ErrorResponseType,
    ConnectionResponseType,
    DiscardCardResponseType,
    PlayCardResponseType,
    HintColorResposeType,
    HintNumberResposeType,
    GameOverResponseType,
}

#[derive(RustcEncodable)]
pub struct ResponseMessage<'n> {
    pub res_type: ResponseType,
    payload:      String,
    next:         Option<&'n str>,
    game_state:   Option<&'n GameState>,
}

impl<'n> ResponseMessage<'n> {
    pub fn new<T>(res_type: ResponseType, payload: &T, next: Option<&'n str>, game_state: Option<&'n GameState>) -> Self
        where T: Encodable
    {
        ResponseMessage {
            res_type:   res_type,
            payload:    json::encode(payload).unwrap(),
            next:       next,
            game_state: game_state,
        }
    }
}

#[derive(RustcEncodable)]
pub struct ErrorResponse {
    explanation: &'static str,
    err_details: Option<String>,
}

impl ErrorResponse {
    pub fn new(explanation: &'static str, err_details: Option<&str>) -> Self {
        ErrorResponse {
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
    name: String
}

impl ConnectionResponse {
    pub fn new<S>(name: S) -> Self
        where S: Into<String> {
        ConnectionResponse {
            name: name.into()
        }
    }
}

#[derive(RustcEncodable)]
pub struct DiscardCardResponse;

#[derive(RustcEncodable)]
pub struct PlayCardResponse;


#[derive(RustcEncodable)]
pub struct HintColorResponse;

#[derive(RustcEncodable)]
pub struct HintNumberResponse;

#[derive(RustcEncodable)]
pub struct GameOverResponse {
    score: usize
}

impl GameOverResponse {
    pub fn new(score: usize) -> Self {
        GameOverResponse {
            score: score
        }
    }
}

pub mod error_messages {
    pub const MSG_TO_TXT_ERROR:              &'static str = "The received message could not be read as a String.";
    pub const UNABLE_TO_DESERIALIZE_MSG:     &'static str = "The received message could not be deserialized";
    pub const UNABLE_TO_DESERIALIZE_PAYLOAD: &'static str = "The payload of the received message could not be deserialized.";
    pub const CATASTROPHIC_FUCKUP:           &'static str = "Catastrophic Fuckup! The server's done goofed.";
    pub const PLAYER_ALREADY_EXISTS:         &'static str = "A Player with the chosen name already exists.";
    pub const ALREADY_CONNECTED:             &'static str = "The Player is already connected.";
    pub const NOT_YET_CONNECTED:             &'static str = "The Player is not yet connected.";
    pub const NO_CARDS:                      &'static str = "The deck has nor more cards.";
    pub const NO_HINT_TOKENS:                &'static str = "Hint token count is zero, a hint cannot be played.";
    pub const CARD_NOT_FOUND:                &'static str = "The given Card cannot be found on the Player's hand.";
    pub const TODO:                          &'static str = "TODO";
}
