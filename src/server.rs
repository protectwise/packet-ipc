use crate::errors::Error;

use crate::packet::{AsIpcPacket, IpcPacket};
use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use log::*;

pub type SenderMessage<'a> = Option<Vec<IpcPacket<'a>>>;
pub type Sender<'a> = IpcSender<SenderMessage<'a>>;

pub struct Server<'a> {
    server: IpcOneShotServer<Sender<'a>>,
    name: String,
}

impl<'a> Server<'a> {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn new() -> Result<Server<'a>, Error> {
        let (server, server_name) = IpcOneShotServer::new().map_err(Error::Io)?;

        Ok(Server {
            server: server,
            name: server_name,
        })
    }

    pub fn accept(self) -> Result<ConnectedIpc<'a>, Error> {
        let (_, tx) = self.server.accept().map_err(Error::Bincode)?;

        info!("Accepted connection from {:?}", tx);

        Ok(ConnectedIpc { connection: tx })
    }
}

pub struct ConnectedIpc<'a> {
    connection: Sender<'a>,
}

impl<'a> ConnectedIpc<'a> {
    pub fn send<T: AsIpcPacket>(&'a self, packets: &'a Vec<T>) -> Result<(), Error> {
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
