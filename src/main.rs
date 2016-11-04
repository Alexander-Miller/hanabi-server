extern crate rand;
extern crate ws;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate env_logger;

mod connection;
mod server;
mod requests;
mod responses;
mod game_state;
mod cards;
mod serialization;
mod display;

use std::rc::Rc;
use std::cell::{Cell, RefCell};
use connection::Connection;
use game_state::GameState;
use server::Server;

fn main() {

    env_logger::init().unwrap();
    info!("test");

    let server = Rc::new(RefCell::new(Server::new(GameState::new(7,3))));
    let token_cell = Cell::new(0);

    // TODO: factory?
    ws::listen("0.0.0.0:4444", |out| {
        let id = token_cell.get();
        token_cell.set(id + 1);
        Connection::new(id, out, server.clone())
    }).unwrap_or_else( |_| {
        error!("Could not open server.");
        ()
    });
}
