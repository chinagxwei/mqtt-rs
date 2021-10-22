#[macro_use]
extern crate lazy_static;

use crate::subscript::Subscript;

pub mod hex;
pub mod tools;
pub mod packet;
pub mod message;
pub mod server;
pub mod client;
pub mod subscript;
pub mod session;


lazy_static! {
    pub static ref SUBSCRIPT: Subscript = Subscript::new();
}

