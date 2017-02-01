use cards::{Color, Number};

#[derive(Debug, PartialEq)]
pub enum RequestType {
    ConnectionRequestType,
    DiscardCardRequestType,
    HintColorRequestType,
    HintNumberRequestType,
    PlayCardRequestType,
    GameStartRequestType,
}

#[derive(RustcDecodable)]
pub struct ConnectionRequest {
    pub msg_type: RequestType,
    pub name:     String
}

#[derive(RustcDecodable)]
pub struct DiscardCardRequest {
    pub msg_type:          RequestType,
    pub discarded_card_id: usize,
}

#[derive(RustcDecodable)]
pub struct HintColorRequest {
    pub msg_type:      RequestType,
    pub target_player: String,
    pub color:         Color,
}

#[derive(RustcDecodable)]
pub struct HintNumberRequest {
    pub msg_type:      RequestType,
    pub target_player: String,
    pub number:        Number,
}

#[derive(RustcDecodable)]
pub struct PlayCardRequest {
    pub msg_type:       RequestType,
    pub played_card_id: usize,
}

#[derive(RustcDecodable)]
pub struct GameStartRequest {
    pub msg_type: RequestType,
}
