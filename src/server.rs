use crate::errors::Error;
use crate::IpcMessage;

use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
use log::*;

pub struct Server {
    server: IpcOneShotServer<IpcSender<Option<IpcMessage>>>,
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
        let (_, tx): (_, IpcSender<Option<IpcMessage>>) =
            self.server.accept().map_err(Error::Bincode)?;

        info!("Accepted connection from {:?}", tx);

        Ok(ConnectedIpc { connection: tx })
    }
}

pub struct ConnectedIpc {
    connection: IpcSender<Option<IpcMessage>>,
}

impl ConnectedIpc {
    pub fn send(&mut self, v: IpcMessage) -> Result<(), Error> {
        self.connection.send(Some(v)).map_err(|e| {
            error!("Failed to send {:?}", e);
            Error::Bincode(e)
        })
    }

    pub fn close(&mut self) -> Result<(), Error> {
        self.connection.send(None).map_err(Error::Bincode)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::packet::{Packet, IpcPacket};
    use ipc_channel::ipc::{self, IpcSender};

    #[test]
    fn test_connection() {
        let server = Server::new().expect("Failed to build server");

        let server_name = server.name().clone();

        let (tx, _rx) = ipc::channel::<Option<IpcMessage>>().expect("Failed to create channel");
        let server_sender: IpcSender<IpcSender<Option<IpcMessage>>> =
            IpcSender::connect(server_name).expect("Server failed to connect");

        let connected_thread = std::thread::spawn(move || {
            server.accept()
        });

        server_sender
            .send(tx)
            .expect("Failed to send client sender");

        connected_thread
            .join()
            .expect("Failed to join")
            .expect("No connection");
    }

    #[test]
    fn test_sending() {
        let server = Server::new().expect("Failed to build server");

        let server_name = server.name().clone();

        let (tx, rx) = ipc::channel::<Option<IpcMessage>>().expect("Failed to create channel");
        let server_sender: IpcSender<IpcSender<Option<IpcMessage>>> =
            IpcSender::connect(server_name).expect("Server failed to connect");

        let connected_thread = std::thread::spawn(move || {
            server.accept()
        });

        server_sender
            .send(tx)
            .expect("Failed to send client sender");

        let mut connection = connected_thread.join().expect("Could not join").expect("No connection");

        let client_result = std::thread::spawn(move || {
            let mut count = 0;
            while let Some(p) = rx.recv().expect("Failed to receive packets") {
                count += p.len();
                if p.is_empty() {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
            count
        });

        connection.send(vec![IpcPacket::try_from(&Packet::new(std::time::SystemTime::now(), vec![2u8])).expect("Failed to serialize")]).expect("Could not send");
        connection.close().expect("Failed to close");

        assert_eq!(client_result.join().expect("Failed to receive"), 1);
    }
}
