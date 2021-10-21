#[macro_use]
extern crate lazy_static;

use crate::subscript::Subscript;

pub mod hex;
pub mod tools;
pub mod packet;
pub mod message;
pub mod server;
pub mod v3_session;
pub mod client;
pub mod subscript;


lazy_static! {
    pub static ref SUBSCRIPT: Subscript = Subscript::new();
}

