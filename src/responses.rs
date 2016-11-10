use rustc_serialize::{json, Encodable};

#[derive(Debug)]
pub enum ResponseType {
    ErrorResponseType,
    ConnectionResponseType,
    DiscardCardResponseType,
    HintColorResposeType,
    HintNumberResposeType,
}

#[derive(RustcEncodable)]
pub struct ResponseMessage {
    pub res_type: ResponseType,
    payload:  String,
}

impl ResponseMessage {
    pub fn new<T>(res_type: ResponseType, payload: &T) -> Self
        where T: Encodable
    {
        ResponseMessage {
            res_type: res_type,
            payload:  json::encode(payload).unwrap(),
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

pub mod error_messages {
    pub const MSG_TO_TXT_ERROR:              &'static str = "The received message could not be read as a String.";
    pub const UNABLE_TO_DESERIALIZE_MSG:     &'static str = "The received message could not be deserailized";
    pub const UNABLE_TO_DESERIALIZE_PAYLOAD: &'static str = "The payload of the received message could not be deserialized.";
    pub const CATASTROPHIC_FUCKUP:           &'static str = "Catastrophic Fuckup! The server's done goofed.";
    pub const PLAYER_ALREADY_EXISTS:         &'static str = "A Player with the chosen name already exists.";
    pub const ALREADY_CONNECTED:             &'static str = "The Player is already connected.";
    pub const NOT_YET_CONNECTED:             &'static str = "The Player is not yet connected.";
    pub const NO_CARDS:                      &'static str = "The deck has nor more cards.";
    pub const CARD_NOT_FOUND:                &'static str = "The given Card cannot be found on the Player's hand.";
    pub const TODO:                          &'static str = "TODO";
}
