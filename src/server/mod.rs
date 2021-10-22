pub mod v3_server;

pub enum ServerHandleKind {
    Response(Vec<u8>),
    Exit(Vec<u8>),
}
