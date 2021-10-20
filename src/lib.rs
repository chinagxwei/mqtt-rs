#[macro_use]
extern crate lazy_static;

use crate::server::Subscript;

pub mod hex;
pub mod tools;
pub mod packet;
pub mod message;
pub mod server;
mod v3_handle;
mod session;


lazy_static! {
    pub static ref SUBSCRIPT: Subscript = Subscript::new();
}

