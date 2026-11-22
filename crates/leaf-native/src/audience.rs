use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::{mpsc, Arc, Mutex};

use leaf_core::network::chat::component::Component;
use leaf_core::network::protocol::game::clientbound_system_chat_packet::ClientboundSystemChatPacket;
use leaf_core::network::protocol::packet::Packet;
use leaf_core::util::uuid::Uuid;

use crate::wire::{PacketIds, Wire};

type Chunk = (i32, i32);
type Message = (Option<Chunk>, Arc<dyn Packet + Send + Sync>);

#[derive(Default)]
pub struct Audience(Mutex<HashMap<Uuid, mpsc::SyncSender<Message>>>);

impl Audience {
    pub fn subscribe(&self, uuid: Uuid) -> Subscription<'_> {
        let (sender, receiver) = mpsc::sync_channel(256);
        self.0.lock().unwrap().insert(uuid, sender);
        Subscription {
            audience: self,
            uuid,
            receiver,
        }
    }

    pub fn broadcast(&self, packet: impl Packet + Send + Sync + 'static, chunk: Option<Chunk>) {
        let packet = Arc::new(packet);
        self.0
            .lock()
            .unwrap()
            .retain(|_, sender| sender.try_send((chunk, packet.clone())).is_ok());
    }

    pub fn chat(&self, content: Component, recipient: Option<Uuid>) {
        let packet = ClientboundSystemChatPacket::new(content, false);
        if let Some(uuid) = recipient {
            let mut audience = self.0.lock().unwrap();
            if audience
                .get(&uuid)
                .is_some_and(|sender| sender.try_send((None, Arc::new(packet))).is_err())
            {
                audience.remove(&uuid);
            }
        } else {
            self.broadcast(packet, None);
        }
    }
}

pub struct Subscription<'a> {
    audience: &'a Audience,
    uuid: Uuid,
    receiver: mpsc::Receiver<Message>,
}

impl Subscription<'_> {
    pub fn flush(
        &self,
        wire: &mut Wire,
        ids: &PacketIds,
        loaded: &HashSet<Chunk>,
    ) -> io::Result<()> {
        loop {
            match self.receiver.try_recv() {
                Ok((chunk, packet)) => {
                    if chunk.is_none_or(|pos| loaded.contains(&pos)) {
                        wire.send(ids, packet.as_ref())?;
                    }
                }
                Err(mpsc::TryRecvError::Empty) => return Ok(()),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(io::Error::other("client update queue is full"))
                }
            }
        }
    }
}

impl Drop for Subscription<'_> {
    fn drop(&mut self) {
        self.audience.0.lock().unwrap().remove(&self.uuid);
    }
}
