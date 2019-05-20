use crate::errors::Error;
use crate::IpcMessage;

use crossbeam_channel::Receiver as CrossbeamReceiver;
use ipc_channel::{
    ipc::{self, IpcSender},
    router::ROUTER,
};

#[derive(Debug)]
pub struct Client {
    receiver: CrossbeamReceiver<Option<IpcMessage>>,
    available: IpcMessage,
    is_closed: bool,
}

impl Client {
    pub fn new(server_name: String) -> Result<Client, Error> {
        let (tx, rx) = ipc::channel::<Option<IpcMessage>>().map_err(Error::Io)?;
        let server_sender = IpcSender::connect(server_name).map_err(Error::Io)?;
        server_sender.send(tx).map_err(Error::Bincode)?;
        let routed_rx =
            ROUTER.route_ipc_receiver_to_new_crossbeam_receiver::<Option<IpcMessage>>(rx);
        Ok(Client {
            receiver: routed_rx,
            available: vec![],
            is_closed: false,
        })
    }

    pub fn take(&mut self, size: usize) -> IpcMessage {
        let packets_to_take = usize::min(size, self.available.len());
        let mut rem = self.available.split_off(packets_to_take);
        std::mem::swap(&mut self.available, &mut rem);
        rem
    }

    pub fn recv(&mut self, size: usize) -> Result<Option<IpcMessage>, Error> {
        if self.is_closed {
            if self.available.is_empty() {
                Ok(None)
            } else {
                Ok(Some(self.take(size)))
            }
        } else if self.available.len() >= size {
            Ok(Some(self.take(size)))
        } else {
            let opt_packets = self.receiver.recv().map_err(Error::Recv)?;
            if let Some(packets) = opt_packets {
                self.available.extend(packets);
            } else {
                self.is_closed = true;
                if self.available.is_empty() {
                    return Ok(None);
                }
            }
            Ok(Some(self.take(size)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ipc_channel::ipc::{IpcOneShotServer, IpcSender};
    use crate::packet::{AsIpcPacket, IpcPacket, Packet};

    #[test]
    fn test_connect_to_server() {
        let (server, server_name) = IpcOneShotServer::new().expect("Failed to create server");

        let server_result = std::thread::spawn(move || {
            let (_, _tx): (_, IpcSender<Option<IpcMessage>>) = server
                .accept()
                .map_err(Error::Bincode)
                .expect("No connection accepted");
        });

        let _client = Client::new(server_name).expect("Failed to connect");

        server_result.join().expect("Thread failed to join");
    }

    #[test]
    fn test_packet_receive() {
        let (server, server_name) = IpcOneShotServer::new().expect("Failed to create server");

        let server_result = std::thread::spawn(move || {
            let (_, tx): (_, IpcSender<Option<IpcMessage>>) = server
                .accept()
                .map_err(Error::Bincode)
                .expect("No connection accepted");

            tx.send(Some(vec![IpcPacket::try_from(&Packet::new(std::time::SystemTime::now(), vec![3u8])).expect("Failed to serialize")]))
                .expect("Failed to send");

            tx.send(None).expect("Failed to send");
        });

        let mut client = Client::new(server_name).expect("Failed to connect");

        let mut packets = client
            .recv(1)
            .expect("Failed to get packets")
            .expect("No packets provided");

        assert_eq!(packets.len(), 1);

        let packets: Vec<_> = packets.drain(..).map(|p| p.into_packet().expect("Could not convert to packet")).collect();
        assert_eq!(packets[0].data()[0], 3u8);

        let addl_packets = client.recv(1).expect("Failed to get packets");

        assert!(addl_packets.is_none());

        server_result.join().expect("Thread failed to join");
    }

    #[test]
    fn test_multiple_packet_receive() {
        let term = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let term_clone = std::sync::Arc::clone(&term);
        let (server, server_name) = IpcOneShotServer::new().expect("Failed to create server");

        let server_result = std::thread::spawn(move || {
            let (_, tx): (_, IpcSender<Option<IpcMessage>>) = server
                .accept()
                .map_err(Error::Bincode)
                .expect("No connection accepted");

            tx.send(Some(vec![IpcPacket::try_from(&Packet::new(std::time::SystemTime::now(), vec![0u8])).expect("Failed to serialize")]))
                .expect("Failed to send");

            while !term_clone.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }

            tx.send(Some(vec![IpcPacket::try_from(&Packet::new(std::time::SystemTime::now(), vec![1u8])).expect("Failed to serialize")]))
                .expect("Failed to send");

            tx.send(None).expect("Failed to send");
        });

        let mut client = Client::new(server_name).expect("Failed to connect");

        let mut packets = client
            .recv(1)
            .expect("Failed to get packets")
            .expect("No packets provided");

        assert_eq!(packets.len(), 1);

        let packets: Vec<_> = packets.drain(..).map(|p| p.into_packet().expect("Could not convert to packet")).collect();
        assert_eq!(packets[0].data()[0], 0u8);

        term.store(true, std::sync::atomic::Ordering::Relaxed);

        let mut packets2 = client
            .recv(1)
            .expect("Failed to get packets")
            .expect("No packets provided");

        assert_eq!(packets2.len(), 1);

        let packets2: Vec<_> = packets2.drain(..).map(|p| p.into_packet().expect("Could not convert to packet")).collect();
        assert_eq!(packets2[0].data()[0], 1u8);

        assert!(client
            .recv(1)
            .expect("Failed to get packets")
            .is_none());

        server_result.join().expect("Thread failed to join");
    }
}
