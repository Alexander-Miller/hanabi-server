use rustc_serialize::{json, Decodable, Encodable};
use ws::{CloseCode, Result, Sender};
use std::error::Error;
use std::collections::HashMap;
use game_state::{CardDrawingResult, CardPlayingResult, GameState, Void};
use connection::Connection;
use requests::RequestType::*;
use requests::{
    RequestMessage,
    ConnectionRequest,
    DiscardCardRequest,
    HintColorRequest,
    HintNumberRequest,
    PlayCardRequest
};
use responses::error_messages::*;
use responses::{
    ResponseMessage,
    ResponseType,
    ErrorResponse,
    ConnectionResponse,
    DiscardCardResponse,
    PlayCardResponse,
    HintColorResponse,
    HintNumberResponse,
    GameOverResponse
};

pub struct Server {
    game_state: GameState,
    finish_count: u8,
    player_map: HashMap<u8, String>,
    connections: Vec<Sender>,
}

impl Server {
    pub fn new(game_state: GameState) -> Self {
        Server {
            game_state:   game_state,
            finish_count: 0,
            player_map:   HashMap::with_capacity(6),
            connections:  Vec::with_capacity(6),
        }
    }

    pub fn answer_with_error_msg(&self, explanation: &'static str, details: Option<&str>, con: &Connection) -> Result<Void> {
        info!("Sending Error Response: '{}'.", explanation);
        let resp_mess = ResponseMessage::new(ResponseType::ErrorResponseType, &ErrorResponse::new(explanation, details));
        let resp_json = json::encode(&resp_mess).expect(CATASTROPHIC_FUCKUP);
        con.out.send(resp_json)
    }

    pub fn answer_with_resp_msg<T>(&self, resp: &T, resp_type: ResponseType, con: &Connection) -> Result<Void>
        where T: Encodable
    {
        info!("Sending Response of type {:?}.", resp_type);
        let resp_json = json::encode(&ResponseMessage::new(resp_type, &resp)).expect(CATASTROPHIC_FUCKUP);
        con.out.broadcast(resp_json)
    }

    fn is_connected(&self, id: u8) -> bool {
        self.player_map.contains_key(&id)
    }

    pub fn handle_req(&mut self, req: &RequestMessage, con: &Connection) -> Result<Void> {
        info!("Received Request: {:?} from Connnection {}.", req.req_type, con.id);
        match (self.is_connected(con.id), req.req_type == ConnectionRequestType) {
            (true, true)   => self.answer_with_error_msg(ALREADY_CONNECTED, None, &con),
            (false, false) => self.answer_with_error_msg(NOT_YET_CONNECTED, None, &con),
            (_ , _)        => {
                match req.req_type {
                    ConnectionRequestType  => self.dispatch_req::<ConnectionRequest>(&req, &con, &mut Self::handle_connection_request),
                    DiscardCardRequestType => self.dispatch_req::<DiscardCardRequest>(&req, &con, &mut Self::handle_discard_request),
                    HintColorRequestType   => self.dispatch_req::<HintColorRequest>(&req, &con, &mut Self::handle_hint_color_request),
                    HintNumberRequestType  => self.dispatch_req::<HintNumberRequest>(&req, &con, &mut Self::handle_hint_number_request),
                    PlayCardRequestType    => self.dispatch_req::<PlayCardRequest>(&req, &con, &mut Self::handle_play_card_request),
                }
            }
        }
    }

    fn dispatch_req<T>(&mut self, req_msg: &RequestMessage, con: &Connection, dispatch_recv: &mut FnMut(&mut Self, &T, &Connection) -> Result<Void>) -> Result<Void>
        where T: Decodable
    {
        debug!("Dispatching Request message type {:?}.", req_msg.req_type);
        match json::decode::<T>(&req_msg.payload) {
            Ok(req) => dispatch_recv(self, &req, &con),
            Err(e)  => self.answer_with_error_msg(UNABLE_TO_DESERIALIZE_PAYLOAD, Some(e.description()), &con)

        }
    }

    fn handle_connection_request(&mut self, req: &ConnectionRequest, con: &Connection) -> Result<Void> {
        info!("Handle Connection Request for player \"{}\" from Connection {}.", req.name, con.id);
        match self.game_state.add_player(self.player_map.get(&con.id).unwrap()) {
            Ok(_) => {
                info!("Connection success.");
                self.player_map.insert(con.id, String::from(req.name.clone()));
                self.connections.push(con.out.clone());
                self.finish_count += 1;
                self.response_dispatch(&ConnectionResponse::new(req.name.as_str()), ResponseType::ConnectionResponseType, &con)
            }
            Err(err_msg) => {
                info!("Connection failure.");
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_discard_request(&mut self, discard_req: &DiscardCardRequest, con: &Connection) -> Result<Void> {
        info!("Handle Discard Request for card \"{}\" from Connection {}.", discard_req.discarded_card, con.id);
        match self.game_state.discard_card(self.player_map.get(&con.id).unwrap(), &discard_req) {
            CardDrawingResult::Ok(deck_is_empty) => {
                info!("Card successfully discarded.");
                if deck_is_empty {
                    self.finish_count -= 1;
                    info!("Deck is empty. Game finishes within {} turns.", self.finish_count);
                }
                self.response_dispatch(&DiscardCardResponse, ResponseType::DiscardCardResponseType, &con)
            }
            CardDrawingResult::Err(err_msg) => {
                info!("Card could not be discarded: {}.", err_msg);
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_hint_color_request(&mut self, hint_color_req: &HintColorRequest, con: &Connection) -> Result<Void> {
        match self.game_state.hint_color(self.player_map.get(&con.id).unwrap(), &hint_color_req) {
            Ok(_)         => self.response_dispatch(&HintColorResponse, ResponseType::HintColorResposeType, &con),
            Err(err_msg)  => self.answer_with_error_msg(err_msg, None, &con),
        }
    }

    fn handle_hint_number_request(&mut self, hint_number_req: &HintNumberRequest, con: &Connection) -> Result<Void> {
        match self.game_state.hint_number(self.player_map.get(&con.id).unwrap(), &hint_number_req) {
            Ok(_)         => self.response_dispatch(&HintNumberResponse, ResponseType::HintNumberResposeType, &con),
            Err(err_msg)  => self.answer_with_error_msg(err_msg, None, &con),
        }
    }

    fn handle_play_card_request(&mut self, play_card_req: &PlayCardRequest, con: &Connection) -> Result<Void> {
        info!("Handle Play Card Request for card \"{}\" from Connection {}.", play_card_req.played_card, con.id);
        match self.game_state.play_card(self.player_map.get(&con.id).unwrap(), &play_card_req) {
            CardPlayingResult::Success => {
                info!("Attempt to play Card {} was successful.", play_card_req.played_card);
                self.response_dispatch(&PlayCardResponse, ResponseType::PlayCardResponseType, &con)
            }
            CardPlayingResult::Failure => {
                info!("Attempt to play Card {} has failed.", play_card_req.played_card);
                self.response_dispatch(&PlayCardResponse, ResponseType::PlayCardResponseType, &con)
            }
            CardPlayingResult::Err(err_msg) => {
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn response_dispatch<T>(&mut self, resp: &T, resp_type: ResponseType, con: &Connection) -> Result<Void>
        where T: Encodable
    {
        if self.game_state.deck_is_empty() {
            self.finish_count -= 1;
            if self.finish_count == 0 {
                return self.game_over(&con);
            }
        }
        self.answer_with_resp_msg(&resp, resp_type, &con)
    }

    fn game_over(&mut self, con: &Connection) -> Result<Void> {
        self.answer_with_resp_msg(&GameOverResponse::new(self.game_state.score()), ResponseType::GameOverResponseType, &con).unwrap();
        for out in &self.connections {
            out.close(CloseCode::Normal).unwrap();
        }
        Ok(())
    }

}
