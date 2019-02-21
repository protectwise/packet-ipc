#![feature(futures_api, async_await, await_macro)]

pub mod client;
pub mod errors;
pub mod packet;
pub mod server;

type IpcMessage = Vec<std::sync::Arc<crate::packet::Packet>>;
