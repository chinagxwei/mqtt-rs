#[macro_use]
extern crate lazy_static;

use crate::server::Subscript;

pub mod hex;
pub mod tools;
pub mod packet;
pub mod message;
pub mod server;


lazy_static! {
    pub static ref SUBSCRIPT: Subscript = Subscript::new();
}

