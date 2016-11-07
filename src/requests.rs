use cards::{Color, Number, Card};

#[derive(Debug, PartialEq)]
pub enum RequestType {
    ConnectionRequestType,
    DiscardCardRequestType,
    HintColorRequestType,
    HintNumberRequestType,
    PlayCardRequestType,
}

#[derive(RustcDecodable)]
pub struct RequestMessage {
    pub req_type: RequestType,
    pub payload:  String,
}

#[derive(RustcDecodable)]
pub struct ConnectionRequest {
    pub name: String
}

#[derive(RustcDecodable)]
pub struct DiscardCardRequest {
    pub discarded_card: Card
}

#[derive(RustcDecodable)]
pub struct HintColorRequest {
    pub target_player: String,
    pub color:         Color,
    pub positive:      bool,
}

#[derive(RustcDecodable)]
pub struct HintNumberRequest {
    pub target_player: String,
    pub number:        Number,
    pub positive:      bool,
}

#[derive(RustcDecodable)]
pub struct PlayCardRequest {
    pub played_card: Card
}
