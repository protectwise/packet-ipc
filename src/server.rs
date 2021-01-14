use crate::errors::Error;

use crate::packet::{AsIpcPacket, IpcPacket};
use ipc_channel::ipc::{IpcOneShotServer, IpcBytesSender};
use log::*;

//pub type SenderMessage<'a> = Option<Vec<IpcPacket<'a>>>;
pub type Sender = IpcBytesSender;

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
    pub fn send<T: AsIpcPacket>(&self, packets: &[T]) -> Result<(), Error> {
        let ipc_packets: Vec<_> = packets
            .iter()
            .map(IpcPacket::from)
            .collect();
        let ipc_packets = Some(ipc_packets);
        let ipc_packets_ser = bincode::serialize(&ipc_packets)
            .map_err(Error::Bincode)?;
        self.connection.send(&ipc_packets_ser).map_err(|e| {
            error!("Failed to send {:?}", e);
            Error::Io(e)
        })
    }

    pub fn close(&mut self) -> Result<(), Error> {
        let none: Option<Vec<IpcPacket>> = None;
        let ser = bincode::serialize(&none).map_err(Error::Bincode)?;

        self.connection.send(&ser).map_err(Error::Io)?;
        Ok(())
    }
}
