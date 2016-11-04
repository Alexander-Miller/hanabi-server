use rustc_serialize::{json, Decodable};
use ws::Result;
use std::error::Error;
use game_state::GameState;
use connection::Connection;
use requests::{
    RequestMessage,
    ConnectionRequest,
    DiscardCardRequest,
    HintColorRequest,
    HintNumberRequest,
    PlayCardRequest
};
use requests::RequestType::*;
use responses::{ResponseMessage, ResponseType, ErrorResponse, ConnectionResponse};
use responses::error_messages::*;

pub struct Server {
    game_state: GameState,
}

impl Server {
    pub fn new(game_state: GameState) -> Self {
        Server {
            game_state: game_state,
        }
    }

    pub fn answer_with_error_msg(&self, explanation: &'static str, details: Option<&str>, con: &Connection) -> Result<()> {
        let resp_mess = ResponseMessage::new(ResponseType::ErrorResponseType, &ErrorResponse::new(explanation, details));
        let resp_json = json::encode(&resp_mess).expect(CATASTROPHIC_FUCKUP);
        con.out.send(resp_json)
    }

    pub fn answer_with_resp_msg(&self, resp_msg: &ResponseMessage, con: &Connection) -> Result<()> {
        info!("Sending Response of type {:?}.", resp_msg.res_type);
        let resp_json = json::encode(&resp_msg).expect(CATASTROPHIC_FUCKUP);
        con.out.broadcast(resp_json)
    }

    pub fn handle_req(&mut self, req: &RequestMessage, con: &Connection) -> Result<()> {
        info!("Received Request: {:?} from Connnection {}.", req.req_type, con.id);
        match req.req_type {
            ConnectionRequestType  => self.dispatch::<ConnectionRequest>(&req, &con, &mut Self::handle_connection_request),
            DiscardCardRequestType => self.dispatch::<DiscardCardRequest>(&req, &con, &mut Self::handle_discard_request),
            HintColorRequestType   => self.dispatch::<HintColorRequest>(&req, &con, &mut Self::handle_hint_color_request),
            HintNumberRequestType  => self.dispatch::<HintNumberRequest>(&req, &con, &mut Self::handle_hint_number_request),
            PlayCardRequestType    => self.dispatch::<PlayCardRequest>(&req, &con, &mut Self::handle_play_card_request),
        }
    }

    fn dispatch<T>(&mut self, req_msg: &RequestMessage, con: &Connection, dispatcher: &mut FnMut(&mut Self, &T, &Connection) -> Result<()>) -> Result<()>
        where T: Decodable
    {
        debug!("Dispatching Request message type {:?}.", req_msg.req_type);
        match json::decode::<T>(&req_msg.payload) {
            Ok(req) => dispatcher(self, &req, con),
            Err(e)  => self.answer_with_error_msg(UNABLE_TO_DESERIALIZE_PAYLOAD, Some(e.description()), &con)

        }
    }

    fn handle_connection_request(&mut self, conn_req: &ConnectionRequest, con: &Connection) -> Result<()> {
        info!("Handle Request for player \"{}\" from Connection {}.", conn_req.name, con.id);
        match self.game_state.add_player(con.id, conn_req.name.as_str()) {
            Ok(_)  => {
                info!("Connection success.");
                let conn_resp = ConnectionResponse::new(conn_req.name.as_str());
                let resp_mess = ResponseMessage::new(ResponseType::ConnectionResponseType, &conn_resp);
                self.answer_with_resp_msg(&resp_mess, &con)
            }
            Err(_) => {
                error!("Connection failure, player already exists.");
                self.answer_with_error_msg(PLAYER_ALREADY_EXISTS, None, &con)
            }
        }
    }

    fn handle_discard_request(&mut self, discard_req: &DiscardCardRequest, con: &Connection) -> Result<()> {
        match self.game_state.discard_card(con.id, &discard_req.discarded_card) {
            Ok(_)        => Ok(()),
            Err(err_msg) => self.answer_with_error_msg(err_msg, None, &con),
        }
    }

    fn handle_hint_color_request(&mut self, hint_color_req: &HintColorRequest, con: &Connection) -> Result<()> {
        match self.game_state.hint_color(&hint_color_req) {
            Ok(_)         => Ok(()),
            Err(err_msg)  => self.answer_with_error_msg(err_msg, None, &con),
        }
    }

    fn handle_hint_number_request(&mut self, hint_number_req: &HintNumberRequest, con: &Connection) -> Result<()> {
        match self.game_state.hint_number(&hint_number_req) {
            Ok(_)         => Ok(()),
            Err(err_msg)  => self.answer_with_error_msg(err_msg, None, &con),
        }
    }

    fn handle_play_card_request(&mut self, play_card_req: &PlayCardRequest, con: &Connection) -> Result<()> {
        Ok(())
    }
}
