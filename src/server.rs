use rustc_serialize::{json, Decodable, Encodable};
use ws::{CloseCode, Result, Sender};
use std::error::Error;
use std::collections::BTreeMap;
use game_state::{CardPlayingResult, GameState, Void};
use connection::Connection;
use requests::RequestType::*;
use requests::{
    RequestMessage,
    ConnectionRequest,
    DiscardCardRequest,
    HintColorRequest,
    HintNumberRequest,
    PlayCardRequest,
    GameStartRequest
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
    GameOverResponse,
    GameStartResponse
};

pub struct Server {
    game_state:   GameState,
    finish_count: usize,
    next_count:   usize,
    player_map:   BTreeMap<u8, String>,
    connections:  Vec<Sender>,
    game_started: bool,
}

impl Server {
    pub fn new(game_state: GameState) -> Self {
        Server {
            game_state:   game_state,
            finish_count: 0,
            next_count:   0,
            player_map:   BTreeMap::new(),
            connections:  Vec::with_capacity(6),
            game_started: false,
        }
    }

    pub fn answer_with_error_msg(&self, explanation: &'static str, details: Option<&str>, con: &Connection) -> Result<Void> {
        info!("Sending Error Response: '{}'.", explanation);
        let resp_mess = ResponseMessage::new(ResponseType::ErrorResponseType, &ErrorResponse::new(explanation, details), None, None);
        let resp_json = json::encode(&resp_mess).expect(CATASTROPHIC_FUCKUP);
        con.out.send(resp_json)
    }

    pub fn answer_with_resp_msg<T>(&mut self, resp: &T, resp_type: ResponseType, include_state: bool, con: &Connection) -> Result<Void>
        where T: Encodable
    {
        info!("Sending Response of type {:?}.", resp_type);
        let (next, state) = match include_state {
            false => {
                debug!("Answer includes no state.");
                (None, None)
            }
            true  => {
                self.next_count = (self.next_count + 1) % self.player_map.keys().len();
                let next = Some(self.player_map.values().nth(self.next_count).as_ref().unwrap().as_str());
                debug!("Answer includes state. Next player is {}.", next.unwrap());
                (next, Some(&self.game_state))
            }
        };
        let resp_json = json::encode(&ResponseMessage::new(resp_type, &resp, next, state)).expect(CATASTROPHIC_FUCKUP);
        con.out.broadcast(resp_json)
    }

    fn is_connected(&self, id: u8) -> bool {
        debug!("Check if id {} is connected.", id);
        self.player_map.contains_key(&id)
    }

    pub fn handle_req(&mut self, req: &RequestMessage, con: &Connection) -> Result<Void> {
        info!("Received Request: {:?} from Connnection {}.", req.req_type, con.id);
        let already_connected = self.is_connected(con.id);
        let is_connecting     = req.req_type == ConnectionRequestType;
        let game_started      = self.game_started;

        if already_connected && is_connecting {
            self.answer_with_error_msg(ALREADY_CONNECTED, None, &con)
        } else if game_started && is_connecting {
            self.answer_with_error_msg(GAME_ALREADY_STARTED, None, &con)
        } else if !already_connected && !is_connecting {
            self.answer_with_error_msg(NOT_YET_CONNECTED, None, &con)
        } else {
            match req.req_type {
                ConnectionRequestType  => self.dispatch_req::<ConnectionRequest>(&req, &con, &mut Self::handle_connection_request),
                DiscardCardRequestType => self.dispatch_req::<DiscardCardRequest>(&req, &con, &mut Self::handle_discard_request),
                HintColorRequestType   => self.dispatch_req::<HintColorRequest>(&req, &con, &mut Self::handle_hint_color_request),
                HintNumberRequestType  => self.dispatch_req::<HintNumberRequest>(&req, &con, &mut Self::handle_hint_number_request),
                PlayCardRequestType    => self.dispatch_req::<PlayCardRequest>(&req, &con, &mut Self::handle_play_card_request),
                GameStartRequestType   => self.dispatch_req::<GameStartRequest>(&req, &con, &mut Self::handle_game_start_request),
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
        match self.game_state.add_player(req.name.as_str()) {
            Ok(_) => {
                info!("Connection success.");
                self.player_map.insert(con.id, String::from(req.name.clone()));
                self.connections.push(con.out.clone());
                self.finish_count += 1;
                self.response_dispatch(&ConnectionResponse::new(req.name.as_str()), ResponseType::ConnectionResponseType, false, &con)
            }
            Err(err_msg) => {
                info!("Connection failure: {}.", err_msg);
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_discard_request(&mut self, discard_req: &DiscardCardRequest, con: &Connection) -> Result<Void> {
        info!("Handle Discard Request for card \"{}\" from Connection {}.", discard_req.discarded_card, con.id);
        match self.game_state.discard_card(self.player_map.get(&con.id).unwrap(), &discard_req) {
            Ok(_) => {
                info!("Card successfully discarded.");
                self.response_dispatch(&DiscardCardResponse, ResponseType::DiscardCardResponseType, true, &con)
            }
            Err(err_msg) => {
                info!("Card could not be discarded: {}.", err_msg);
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_hint_color_request(&mut self, hint_color_req: &HintColorRequest, con: &Connection) -> Result<Void> {
        info!("Handle Hint Color Request for color \"{}\" from Connection {}.", hint_color_req.color, con.id);
        match self.game_state.hint_color(self.player_map.get(&con.id).unwrap(), &hint_color_req) {
            Ok(_)         => self.response_dispatch(&HintColorResponse, ResponseType::HintColorResposeType, true, &con),
            Err(err_msg)  => self.answer_with_error_msg(err_msg, None, &con),
        }
    }

    fn handle_hint_number_request(&mut self, hint_number_req: &HintNumberRequest, con: &Connection) -> Result<Void> {
        info!("Handle Hint Number Request for color \"{}\" from Connection {}.", hint_number_req.number, con.id);
        match self.game_state.hint_number(self.player_map.get(&con.id).unwrap(), &hint_number_req) {
            Ok(_)         => self.response_dispatch(&HintNumberResponse, ResponseType::HintNumberResposeType, true, &con),
            Err(err_msg)  => self.answer_with_error_msg(err_msg, None, &con),
        }
    }

    fn handle_play_card_request(&mut self, play_card_req: &PlayCardRequest, con: &Connection) -> Result<Void> {
        info!("Handle Play Card Request for card \"{}\" from Connection {}.", play_card_req.played_card, con.id);
        match self.game_state.play_card(self.player_map.get(&con.id).unwrap(), &play_card_req) {
            CardPlayingResult::Success => {
                info!("Attempt to play Card {} was successful.", play_card_req.played_card);
                self.response_dispatch(&PlayCardResponse, ResponseType::PlayCardResponseType, true, &con)
            }
            CardPlayingResult::Failure => {
                info!("Attempt to play Card {} has failed.", play_card_req.played_card);
                self.response_dispatch(&PlayCardResponse, ResponseType::PlayCardResponseType, true, &con)
            }
            CardPlayingResult::EpicFail => {
                info!("Attempt to play Card {} has failed and all error tokens are used up.", play_card_req.played_card);
                self.game_over(&con)
            }
            CardPlayingResult::Err(err_msg) => {
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_game_start_request(&mut self, _: &GameStartRequest, con: &Connection) -> Result<Void> {
        info!("Starting game.");
        self.game_started = true;
        self.response_dispatch(&GameStartResponse, ResponseType::GameStartResponseType, true, &con)
    }

    fn response_dispatch<T>(&mut self, resp: &T, resp_type: ResponseType, include_state: bool, con: &Connection) -> Result<Void>
        where T: Encodable
    {
        debug!("Dispatching reponse of type: {}, with included state: {} for connection {}.", resp_type, include_state, con.id);
        if self.game_state.deck_is_empty() {
            self.finish_count -= 1;
            debug!("Deck is empty. {} turns left till game over.", self.finish_count);
            if self.finish_count == 0 {
                return self.game_over(&con);
            }
        }
        self.answer_with_resp_msg(&resp, resp_type, include_state, &con)
    }

    fn game_over(&mut self, con: &Connection) -> Result<Void> {
        let score = self.game_state.score();
        info!("Game Over! Final score: {}.", score);
        self.answer_with_resp_msg(&GameOverResponse::new(score), ResponseType::GameOverResponseType, false, &con).unwrap();
        for out in &self.connections {
            out.close(CloseCode::Normal).unwrap();
        }
        Ok(())
    }

}
