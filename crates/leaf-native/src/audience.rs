use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::{mpsc, Arc, Mutex};

use leaf_core::network::chat::component::Component;
use leaf_core::network::protocol::game::clientbound_system_chat_packet::ClientboundSystemChatPacket;
use leaf_core::network::protocol::packet::Packet;
use leaf_core::util::uuid::Uuid;

use crate::wire::{PacketIds, Wire};

type Chunk = (i32, i32);
type ScopedPacket = (Option<Chunk>, Arc<dyn Packet + Send + Sync>);

#[derive(Clone)]
enum Message {
    Single(ScopedPacket),
    Batch(Arc<[ScopedPacket]>),
}

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
        self.broadcast_message(Message::Single((chunk, Arc::new(packet))));
    }

    pub fn broadcast_batch<P>(&self, packets: impl IntoIterator<Item = (Chunk, P)>)
    where
        P: Packet + Send + Sync + 'static,
    {
        let packets = packets
            .into_iter()
            .map(|(chunk, packet)| {
                (
                    Some(chunk),
                    Arc::new(packet) as Arc<dyn Packet + Send + Sync>,
                )
            })
            .collect::<Arc<[_]>>();
        if !packets.is_empty() {
            self.broadcast_message(Message::Batch(packets));
        }
    }

    fn broadcast_message(&self, message: Message) {
        self.0
            .lock()
            .unwrap()
            .retain(|_, sender| sender.try_send(message.clone()).is_ok());
    }

    pub fn chat(&self, content: Component, recipient: Option<Uuid>) {
        let packet = ClientboundSystemChatPacket::new(content, false);
        if let Some(uuid) = recipient {
            self.send(uuid, packet);
        } else {
            self.broadcast(packet, None);
        }
    }

    pub fn send(&self, recipient: Uuid, packet: impl Packet + Send + Sync + 'static) {
        let mut audience = self.0.lock().unwrap();
        if audience.get(&recipient).is_some_and(|sender| {
            sender
                .try_send(Message::Single((None, Arc::new(packet))))
                .is_err()
        }) {
            audience.remove(&recipient);
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
                Ok(message) => {
                    let packets = match &message {
                        Message::Single(packet) => std::slice::from_ref(packet),
                        Message::Batch(packets) => packets.as_ref(),
                    };
                    for (chunk, packet) in packets {
                        if chunk.is_none_or(|pos| loaded.contains(&pos)) {
                            wire.send(ids, packet.as_ref())?;
                        }
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
