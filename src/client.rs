use crate::errors::Error;

use crate::packet::{IpcPacket, Packet};
use crossbeam_channel::{Receiver as CrossbeamReceiver, Sender as CrossbeamSender};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::ipc::{IpcReceiverSet, IpcSelectionResult};
use log::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct Client {
    receiver: CrossbeamReceiver<Option<Vec<Arc<Packet>>>>,
    available: Vec<Arc<Packet>>,
    is_closed: bool,
}

fn process_selection_result(
    msg_tx: &CrossbeamSender<Option<Vec<Arc<Packet>>>>,
    result: IpcSelectionResult,
) -> bool {
    let mut closed = false;
    match result {
        IpcSelectionResult::MessageReceived(_id, message) => {
            let opt_packets = match message.to::<Option<Vec<Packet>>>() {
                Err(e) => {
                    error!("Failed to convert message to packets: {:?}", e);
                    None
                }
                Ok(opt_packets) => opt_packets.map(|packets| {
                    let packets: Vec<_> = packets.into_iter().map(|p| Arc::new(p)).collect();
                    packets
                }),
            };
            closed = opt_packets.is_none();
            if let Err(e) = msg_tx.send(opt_packets) {
                error!("Failed to send message: {:?}", e);
                closed = true;
            }
        }
        IpcSelectionResult::ChannelClosed(_id) => {
            if let Err(e) = msg_tx.send(None) {
                error!("Failed to send message: {:?}", e);
                closed = true;
            }
        }
    }
    closed
}

impl Client {
    /// Uses a unbounded channel to transfer data.
    pub fn new(server_name: String) -> Result<Client, Error> {
        Self::new_with_size(server_name, None)
    }
    /// new client with a choice of bounded or unbounded based on the channel_size bening Some(size) or None
    pub fn new_with_size(
        server_name: String,
        channel_size: Option<usize>,
    ) -> Result<Client, Error> {
        let (ipc_tx, ipc_rx) = ipc::channel::<IpcSender<Vec<IpcPacket>>>().map_err(Error::Io)?;
        let server_sender = IpcSender::connect(server_name).map_err(Error::Io)?;
        server_sender.send(ipc_tx).map_err(Error::Bincode)?;

        let mut receiver = IpcReceiverSet::new().map_err(Error::Io)?;
        receiver.add_opaque(ipc_rx.to_opaque()).map_err(Error::Io)?;

        let (msg_tx, msg_rx) = match channel_size {
            Some(channel_size) => crossbeam_channel::bounded(channel_size),
            None => crossbeam_channel::unbounded(),
        };

        std::thread::spawn(move || {
            let mut closed = false;
            while !closed {
                match receiver.select() {
                    Err(e) => {
                        error!("Failed to receive packets: {:?}", e);
                        closed = true;
                    }
                    Ok(results) => {
                        for result in results.into_iter() {
                            closed = closed || process_selection_result(&msg_tx, result);
                        }
                    }
                }
            }
        });
        Ok(Client {
            receiver: msg_rx,
            available: vec![],
            is_closed: false,
        })
    }

    pub fn take(&mut self, size: usize) -> Vec<Arc<Packet>> {
        let packets_to_take = usize::min(size, self.available.len());
        let mut rem = self.available.split_off(packets_to_take);
        std::mem::swap(&mut self.available, &mut rem);
        rem
    }

    pub fn recv(&mut self, size: usize) -> Result<Option<Vec<Arc<Packet>>>, Error> {
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
