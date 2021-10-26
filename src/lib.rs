#[macro_use]
extern crate lazy_static;

use crate::subscript::Subscript;
use crate::container::MessageContainer;

pub mod hex;
pub mod tools;
pub mod packet;
pub mod message;
pub mod server;
pub mod client;
pub mod subscript;
pub mod session;
pub mod container;


lazy_static! {
    pub static ref SUBSCRIPT: Subscript = Subscript::new();
    pub static ref MESSAGE_CONTAINER: MessageContainer = MessageContainer::new();
}

