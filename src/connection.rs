use ws::{Handler, Message, Sender, Handshake, Result, CloseCode, Error};
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error as StdError;
use server::Server;
use responses::error_messages::MSG_TO_TXT_ERROR;
use game_state::Void;

pub struct Connection {
    pub id:  u8,
    pub out: Sender,
    server:  Rc<RefCell<Server>>
}

impl Handler for Connection {
    fn on_open(&mut self, _: Handshake) -> Result<Void> {
        info!("On Open.");
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("On Close with CloseCode '{:?}' and Reason '{}'.", code ,reason);
    }

    fn on_error(&mut self, err: Error) {
        error!("On Error with Error '{}'.", err);
    }

    fn on_message(&mut self, msg: Message) -> Result<Void> {
        info!("On message.");
        match msg.as_text() {
            Ok(txt) => self.dispatch_req_to_server(&txt),
            Err(e)  => self.dispatch_err_to_server(MSG_TO_TXT_ERROR, Some(e.description())),
        }
    }
}

impl Connection {

    pub fn new(id: u8, out: Sender, server: Rc<RefCell<Server>>) -> Self {
        debug!("Creating new Connection instace with id {}", id);
        Connection {
            id:     id,
            out:    out,
            server: server,
        }
    }

    fn dispatch_err_to_server(&self, explanation: &'static str, details: Option<&str>) -> Result<Void> {
        debug!("Dispatch err to server: \"{}\"", explanation);
        self.server.borrow().answer_with_error_msg(&explanation, details, &self)
    }

    fn dispatch_req_to_server(&self, req: &str) -> Result<Void> {
       debug!("Dispath Request to server.");
       self.server.borrow_mut().handle_req(&req, &self)
    }
}
