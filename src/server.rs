use crate::errors::Error;

use crate::packet::{AsIpcPacket, IpcPacket};
use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use log::*;
use std::sync::{Arc, Mutex};

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

        let tx = Arc::new(Mutex::new(tx));

        Ok(ConnectedIpc { connection: tx })
    }
}


pub struct ConnectedIpc {
    connection: Arc<Mutex<Sender>>,
}

impl ConnectedIpc {
    pub async fn send<T: AsIpcPacket>(&self, packets:Vec<T>) -> Result<(), Error> {
        let ipc_packets: Vec<_> = packets.iter().map(IpcPacket::from).collect();
        Self::internal_send(Arc::clone(&self.connection), ipc_packets).await
    }

    async fn internal_send(sender: Arc<Mutex<Sender>>, ipc_packets: Vec<IpcPacket>) -> Result<(), Error> {
        blocking::Unblock::new(()).with_mut(move |_| {
            let sender = Arc::clone(&sender);
            let sender = sender.lock().map_err(|_| Error::Mutex())?;
            sender.send(Some(ipc_packets)).map_err(|e| {
                error!("Failed to send {:?}", e);
                Error::Bincode(e)
            });
            Ok(())
        }).await

    }

    pub fn close(&mut self) -> Result<(), Error> {
        let connection = Arc::clone(&self.connection);
        let connection = connection.lock().map_err(|_| Error::Mutex())?;
        connection.send(None).map_err(Error::Bincode)?;
        Ok(())
    }
}
