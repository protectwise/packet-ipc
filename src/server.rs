use crate::errors::{
    Error,
    ErrorKind
};

use crate::packet::{
    AsIpcPacket,
    Packet
};

use futures::{
    Async,
    Future,
    Poll,
    Stream
};
use ipc_channel::ipc::{
    IpcOneShotServer,
    IpcSender
};
use log::*;

pub struct Server {
    server: IpcOneShotServer<IpcSender<Option<Vec<Packet>>>>,
    name: String
}

impl Server {
    pub fn name(&self) -> &String { &self.name }

    pub fn new() -> Result<Server, Error> {
        let (server, server_name) = IpcOneShotServer::new()?;

        Ok(Server {
            server: server,
            name: server_name
        })
    }

    pub fn accept(self) -> impl Future<Item=ConnectedIpc, Error=Error> {
        futures::lazy(|| {
            let (_, tx): (_, IpcSender<Option<Vec<Packet>>>) = self.server.accept().map_err(|_| {
                Error::from_kind(ErrorKind::IpcFailure)
            })?;

            info!("Accepted connection from {:?}", tx);

            Ok(ConnectedIpc {
                connection: tx
            })
        })
    }
}

pub struct ConnectedIpc {
    connection: IpcSender<Option<Vec<Packet>>>
}

impl ConnectedIpc {
    pub fn send(&mut self, packets: Vec<Packet>) -> Result<(), Error> {
        self.connection.send(Some(packets)).map_err(|e| {
            error!("Failed to send {:?}", e);
            Error::from_kind(ErrorKind::Bincode)
        })
    }

    pub fn close(&mut self) -> Result<(), Error> {
        info!("Closing IPC Server");
        self.connection.send(None).map_err(|_| {
            Error::from_kind(ErrorKind::Bincode)
        })
    }
}

pub struct IpcTransfer<T> {
    inner: T,
    ipc: ConnectedIpc
}

impl<T> Stream for IpcTransfer<T>
    where T: Stream,
          T::Item: IntoIterator,
          <<T as Stream>::Item as IntoIterator>::Item: AsIpcPacket,
          T::Error: From<Error>
{
    type Item=Vec<<<T as Stream>::Item as IntoIterator>::Item>;
    type Error=T::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.inner.poll()? {
            Async::NotReady => {
                debug!("No packets to send to ipc");
                Ok(Async::NotReady)
            },
            Async::Ready(None) => {
                self.ipc.close().map_err(Self::Error::from)?;
                Ok(Async::Ready(None))
            }
            Async::Ready(Some(packets)) => {
                let iter = packets.into_iter();
                let mut packets = vec![];
                let to_send = iter.map(|p| {
                    let ipc = p.as_ipc_packet();
                    packets.push(p);
                    ipc
                }).collect();
                self.ipc.send(to_send).map_err(Self::Error::from)?;
                Ok(Async::Ready(Some(packets)))
            }
        }
    }
}

pub trait WithIpcTransfer: Stream {
    fn transfer_ipc(
        self,
        ipc: ConnectedIpc
    ) -> IpcTransfer<Self>
        where Self: Sized
    {
        IpcTransfer {
            inner: self,
            ipc: ipc
        }
    }
}

impl<T: ?Sized> WithIpcTransfer for T
    where T: Stream
{}

#[cfg(test)]
mod tests {
    use super::*;

    use ipc_channel::{
        ipc::{
            self,
            IpcSender
        }
    };

    #[test]
    fn test_connection() {
        let server = Server::new().expect("Failed to build server");

        let server_name = server.name().clone();

        let future_accept = server.accept();

        let (tx, rx) = ipc::channel::<Option<Vec<Packet>>>().expect("Failed to create channel");
        let server_sender: IpcSender<Option<Vec<Packet>>> = IpcSender::connect(server_name).expect("Server failed to connect");
    }
}

