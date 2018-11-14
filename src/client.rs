use crate::errors::{Error, ErrorKind};
use crate::packet::Packet;

use crossbeam_channel::Receiver as CrossbeamReceiver;
use ipc_channel::{
    router::ROUTER,
    ipc::{
        self,
        IpcSender
    }
};

#[derive(Debug)]
pub struct Client {
    receiver: CrossbeamReceiver<Option<Vec<Packet>>>,
    available_packets: Vec<Packet>,
    is_closed: bool
}

impl Client {
    pub fn new(server_name: String) -> Result<Client, Error> {
        let (tx, rx) = ipc::channel::<Option<Vec<Packet>>>().map_err(Error::from)?;
        let server_sender = IpcSender::connect(server_name).map_err(Error::from)?;
        server_sender.send(tx).map_err(|_| Error::from_kind(ErrorKind::Bincode))?;
        let routed_rx = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver::<Option<Vec<Packet>>>(rx);
        Ok(Client {
            receiver: routed_rx,
            available_packets: vec![],
            is_closed: false
        })
    }

    fn take_packets(&mut self, size: usize) -> Vec<Packet> {
        let packets_to_take = usize::min(size, self.available_packets.len());
        let mut rem = self.available_packets.split_off(packets_to_take);
        std::mem::swap(&mut self.available_packets, &mut rem);
        rem
    }

    pub fn receive_packets(&mut self, size: usize) -> Result<Option<Vec<Packet>>, Error> {
        if self.is_closed {
            if self.available_packets.is_empty() {
                Ok(None)
            } else {
                Ok(Some(self.take_packets(size)))
            }
        } else if self.available_packets.len() >= size {
            Ok(Some(self.take_packets(size)))
        } else {
            let opt_packets = self.receiver.recv()?;
            if let Some(packets) = opt_packets {
                self.available_packets.extend(packets);
            } else {
                self.is_closed = true;
            }
            Ok(Some(self.take_packets(size)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_to_server() {

    }

    #[test]
    fn test_packet_receive() {

    }
}