pub mod client;
pub mod errors;
pub mod packet;
pub mod server;

type IpcMessage = Vec<packet::IpcPacket>;
