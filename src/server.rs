use crate::errors::Error;

use crate::packet::{AsIpcPacket, IpcPacket};
use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use log::*;

pub type SenderMessage = Option<Vec<IpcPacket>>;
pub type Sender = IpcSender<SenderMessage>;

pub struct Server {
    server: IpcOneShotServer<Sender>,
    name: String,
}

impl Server {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn new() -> Result<Server, Error> {
        let (server, server_name) = IpcOneShotServer::new().map_err(Error::Io)?;

        Ok(Server {
            server: server,
            name: server_name,
        })
    }

    pub fn accept(self) -> Result<ConnectedIpc, Error> {
        let (_, tx) = self.server.accept().map_err(Error::Bincode)?;

        info!("Accepted connection from {:?}", tx);

        Ok(ConnectedIpc { connection: tx })
    }
}

pub struct ConnectedIpc {
    connection: Sender,
}

impl ConnectedIpc {
    pub fn send<T: AsIpcPacket>(&self, packets: Vec<T>) -> Result<(), Error> {
        let ipc_packets: Vec<_> = packets.iter().map(IpcPacket::from).collect();
        self.connection.send(Some(ipc_packets)).map_err(|e| {
            error!("Failed to send {:?}", e);
            Error::Bincode(e)
        })
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.connection.send(None).map_err(Error::Bincode)?;
        Ok(())
    }
}
