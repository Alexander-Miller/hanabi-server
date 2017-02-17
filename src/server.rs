use rustc_serialize::{json, Decodable, Encodable};
use rustc_serialize::json::Json;
use ws::{CloseCode, Result, Sender};
use std::error::Error;
use std::collections::BTreeMap;
use std::result::Result as StdResult;
use game_state::{CardPlayingResult, GameState, Void, DiscardCardResult};
use connection::Connection;
use requests::RequestType::*;
use requests::{
    RequestType,
    ConnectionRequest,
    DiscardCardRequest,
    HintColorRequest,
    HintNumberRequest,
    PlayCardRequest,
    GameStartRequest
};
use responses::error_messages::*;
use responses::{
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
    player_map:   BTreeMap<u8, String>,
    connections:  Vec<Sender>,
    game_started: bool,
}

impl Server {
    pub fn new(game_state: GameState) -> Self {
        debug!("Creating new server instance.");
        Server {
            game_state:   game_state,
            player_map:   BTreeMap::new(),
            connections:  Vec::with_capacity(6),
            game_started: false,
        }
    }

    pub fn answer_with_error_msg(&self, explanation: &'static str, details: Option<&str>, con: &Connection) -> Result<Void> {
        info!("Sending Error Response: '{}'.", explanation);
        let resp_json = json::encode(&ErrorResponse::new(explanation, details)).expect(CATASTROPHIC_FUCKUP);
        con.out.send(resp_json)
    }

    fn encode_response<T>(&self, resp: &T) -> String
        where T: Encodable
    {
        debug!("Encoding Response.");
        json::encode(&resp).expect(CATASTROPHIC_FUCKUP)
    }

    fn is_connected(&self, id: u8) -> bool {
        debug!("Check if id {} is connected.", id);
        self.player_map.contains_key(&id)
    }

    pub fn handle_req(&mut self, req: &str, con: &Connection) -> Result<Void> {

        fn get_req_type(req: &str) -> StdResult<RequestType, Void> {
            let req_json = match Json::from_str(&req) {
                Ok(js) => js,
                Err(_) => return Err(())
            };
            let req_obj = match req_json.as_object() {
                Some(obj) => obj,
                None      => return Err(())
            };
            let type_json = match req_obj.get("msg_type") {
                Some(json) => json,
                None       => return Err(()),
            };
            let type_str = match type_json.as_string() {
                Some(t_str) => t_str,
                None        => return Err(()),
            };
            match json::decode::<RequestType>(format!("\"{}\"", type_str).as_str()) {
                Ok(req_type) => Ok(req_type),
                Err(_)       => Err(())
            }
        }

        if self.player_map.get(&con.id).unwrap() != self.game_state.get_next_player() && !self.game_state.get_next_player().is_empty() {
            return self.answer_with_error_msg(NOT_YOUR_TURN, None, &con)
        }


        let req_type = match get_req_type(&req) {
            Ok(t)  => t,
            Err(_) => return self.answer_with_error_msg(UNABLE_TO_GET_MSG_TYPE, None, &con)
        };

        info!("Received Request of type {:?} from Connection {}.", req_type, con.id);
        let already_connected = self.is_connected(con.id);
        let is_connecting     = req_type == ConnectionRequestType;

        if already_connected && is_connecting {
            self.answer_with_error_msg(ALREADY_CONNECTED, None, &con)
        } else if self.game_started && is_connecting {
            self.answer_with_error_msg(CONN_GAME_ALREADY_STARTED, None, &con)
        } else if !already_connected && !is_connecting {
            self.answer_with_error_msg(NOT_YET_CONNECTED, None, &con)
        } else {
            match req_type {
                ConnectionRequestType  => self.dispatch_req::<ConnectionRequest>(&req, &con, &mut Self::handle_connection_request),
                DiscardCardRequestType => self.dispatch_req::<DiscardCardRequest>(&req, &con, &mut Self::handle_discard_request),
                HintColorRequestType   => self.dispatch_req::<HintColorRequest>(&req, &con, &mut Self::handle_hint_color_request),
                HintNumberRequestType  => self.dispatch_req::<HintNumberRequest>(&req, &con, &mut Self::handle_hint_number_request),
                PlayCardRequestType    => self.dispatch_req::<PlayCardRequest>(&req, &con, &mut Self::handle_play_card_request),
                GameStartRequestType   => self.dispatch_req::<GameStartRequest>(&req, &con, &mut Self::handle_game_start_request),
            }
        }
    }

    fn dispatch_req<T>(&mut self, req_str: &str, con: &Connection, dispatch_recv: &mut FnMut(&mut Self, &T, &Connection) -> Result<Void>) -> Result<Void>
        where T: Decodable
    {
        debug!("Dispatching Request.");
        match json::decode::<T>(&req_str) {
            Ok(req) => dispatch_recv(self, &req, &con),
            Err(e)  => self.answer_with_error_msg(UNABLE_TO_DESERIALIZE_PAYLOAD, Some(e.description()), &con)
        }
    }

    fn handle_connection_request(&mut self, req: &ConnectionRequest, con: &Connection) -> Result<Void> {
        info!("Handle Connection Request for player {} from Connection {}.", req.name, con.id);
        match self.game_state.add_player(req.name.as_str()) {
            Ok(_) => {
                info!("Connection success.");
                self.player_map.insert(con.id, String::from(req.name.clone()));
                self.connections.push(con.out.clone());
                let response = &self.encode_response(&ConnectionResponse::new(self.player_map.values().map(|n| n.as_str()).collect::<Vec<&str>>()));
                self.answer_with_resp_msg(response, &con)
            }
            Err(err_msg) => {
                error!("Connection failure: {}.", err_msg);
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_discard_request(&mut self, discard_req: &DiscardCardRequest, con: &Connection) -> Result<Void> {
        info!("Handle Discard Request for card #{} from Connection {}.", discard_req.discarded_card_id, con.id);
        let player = self.player_map.get(&con.id).unwrap();
        match self.game_state.discard_card(player, discard_req.discarded_card_id) {
            DiscardCardResult::Ok{discarded_card: discarded, drawn_card: drawn} => {
                info!("Card #{} successfully discarded.", discard_req.discarded_card_id);
                let response = &self.encode_response(&DiscardCardResponse::new(player, &discarded, drawn.as_ref(), &self.game_state));
                self.answer_with_resp_msg(response, &con)
            }
            DiscardCardResult::Err(err_msg) => {
                error!("Card #{} could not be discarded: {}.", discard_req.discarded_card_id, err_msg);
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_hint_color_request(&mut self, hint_color_req: &HintColorRequest, con: &Connection) -> Result<Void> {
        info!("Handle Hint Color Request for color {} from Connection {} for player {}.", hint_color_req.color, con.id, hint_color_req.target_player);
        let player = self.player_map.get(&con.id).unwrap();
        match self.game_state.hint_color(&hint_color_req.target_player, &hint_color_req.color) {
            Ok(_) => {
                info!("Color hint for player {} successful", hint_color_req.target_player);
                let response = &self.encode_response(&HintColorResponse::new(
                    player, &hint_color_req.target_player, &hint_color_req.color, &self.game_state));
                self.answer_with_resp_msg(response, &con)
            }
            Err(err_msg) => {
                error!("Color hint could not be given to player {}: {}", hint_color_req.target_player, err_msg);
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_hint_number_request(&mut self, hint_number_req: &HintNumberRequest, con: &Connection) -> Result<Void> {
        info!("Handle Hint Number Request for color {} from Connection {}.", hint_number_req.number, con.id);
        let player = self.player_map.get(&con.id).unwrap();
        match self.game_state.hint_number(&hint_number_req.target_player, &hint_number_req.number) {
            Ok(_) => {
                info!("Number hint for player {} successful", hint_number_req.target_player);
                let response = &self.encode_response(&HintNumberResponse::new(
                    player, &hint_number_req.target_player, &hint_number_req.number, &self.game_state));
                self.answer_with_resp_msg(response, &con)
            }
            Err(err_msg) => {
                error!("Number hint could not be given to player {}: {}", hint_number_req.target_player, err_msg);
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_play_card_request(&mut self, play_card_req: &PlayCardRequest, con: &Connection) -> Result<Void> {
        info!("Handle Play Card Request for card #{} from Connection {}.", play_card_req.played_card_id, con.id);
        let player = self.player_map.get(&con.id).unwrap();
        match self.game_state.play_card(player, play_card_req.played_card_id) {
            CardPlayingResult::Ok {
                success,
                played_card,
                drawn_card,
            } => {
                match success {
                    true  => info!("Attempt of player {} to play card #{} was successful.", player, play_card_req.played_card_id),
                    false => info!("Attempt of player {} to play card #{} has failed.", player, play_card_req.played_card_id),
                }
                let response = &self.encode_response(
                    &PlayCardResponse::new(&player, &played_card, drawn_card.as_ref(), success, &self.game_state));
                self.answer_with_resp_msg(response, &con)
            }
            CardPlayingResult::EpicFail => {
                info!("Attempt to play Card #{} has failed and all error tokens are used up.", play_card_req.played_card_id);
                self.game_over(&con)
            }
            CardPlayingResult::Err(err_msg) => {
                error!("Error when player {} tried to play card #{}", player, play_card_req.played_card_id);
                self.answer_with_error_msg(err_msg, None, &con)
            }
        }
    }

    fn handle_game_start_request(&mut self, _: &GameStartRequest, con: &Connection) -> Result<Void> {
        if self.game_started {
            error!("Received request to start game after it was started already.");
            self.answer_with_error_msg(GAME_ALREADY_STARTED, None, &con)
        } else {
            info!("Starting game.");
            self.game_started = true;
            let response = &self.encode_response(&GameStartResponse::new(&self.game_state));
            self.answer_with_resp_msg(response, &con)
        }
    }

    fn answer_with_resp_msg(&self, resp: &str, con: &Connection) -> Result<Void> {
        debug!("Dispatching reponse for connection {}.", con.id);
        if let Some(0) = self.game_state.turns_left() {
            return self.game_over(&con);
        }
        con.out.broadcast(resp)
    }

    fn game_over(&self, con: &Connection) -> Result<Void> {
        let score = self.game_state.score();
        info!("Game Over! Final score: {}.", score);
        let response = self.encode_response(&GameOverResponse::new(score));
        con.out.broadcast(response.as_str()).unwrap();
        for out in &self.connections {
            out.close(CloseCode::Normal).unwrap();
        }
        Ok(())
    }

}
