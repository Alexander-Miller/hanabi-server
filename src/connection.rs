use ws::{Handler, Message, Sender, Handshake, Result, CloseCode, Error};
use rustc_serialize::json;
use std::rc::Rc;
use std::cell::RefCell;
use std::error::Error as StdError;
use server::Server;
use responses::error_messages::{MSG_TO_TXT_ERROR, UNABLE_TO_DESERIALIZE_MSG};
use requests::RequestMessage;

pub struct Connection {
    pub id:  u8,
    pub out: Sender,
    server:  Rc<RefCell<Server>>
}

impl Handler for Connection {
    fn on_open(&mut self, hs: Handshake) -> Result<()> {
        info!("On Open with Handshake '{:?}'.", hs);
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        info!("On Close with CloseCode '{:?}' and Reason '{}'.", code ,reason);
    }

    fn on_error(&mut self, err: Error) {
        error!("On Error with Error '{}'.", err);
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        info!("On message with Message '{}'", msg);
        match msg.as_text() {
            Ok(txt) => {
                match json::decode::<RequestMessage>(&txt) {
                    Ok(req) => self.dispatch_req_to_server(&req),
                    Err(e)  => self.dispatch_err_to_server(UNABLE_TO_DESERIALIZE_MSG, Some(e.description())),
                }
            }
            Err(e)  => self.dispatch_err_to_server(MSG_TO_TXT_ERROR, Some(e.description())),
        }
    }
}

impl Connection {

    pub fn new(id: u8, out: Sender, server: Rc<RefCell<Server>>) -> Self {
        debug!("New Connection instace with id {}", id);
        Connection {
            id:     id,
            out:    out,
            server: server,
        }
    }

    fn dispatch_err_to_server(&self, explanation: &'static str, details: Option<&str>) -> Result<()> {
        debug!("Dispatch err to server: \"{}\"", explanation);
        self.server.borrow().answer_with_error_msg(&explanation, details, &self)
    }

    fn dispatch_req_to_server(&self, req: &RequestMessage) -> Result<()> {
       debug!("Dispath Request to server: {:?}", req.req_type);
       self.server.borrow_mut().handle_req(&req, &self)
    }
}
