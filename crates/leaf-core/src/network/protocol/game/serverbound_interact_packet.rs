//! Перенос `ServerboundInteractPacket` (+ `ActionType`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::interaction_hand::InteractionHand;
use crate::world::phys::vec3::Vec3;

/// `ServerboundInteractPacket.ActionType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractActionType {
    /// `INTERACT`.
    Interact = 0,
    /// `ATTACK`.
    Attack = 1,
    /// `INTERACT_AT`.
    InteractAt = 2,
}

impl InteractActionType {
    /// `values()`-порядок.
    pub const VALUES: [InteractActionType; 3] = [
        InteractActionType::Interact,
        InteractActionType::Attack,
        InteractActionType::InteractAt,
    ];

    /// `byId` — ordinal.
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(3) as usize]
    }

    /// `ordinal`.
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// `ServerboundInteractPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractAction {
    /// `INTERACT` — рука.
    Interaction(InteractionHand),
    /// `ATTACK` — без данных.
    Attack,
    /// `INTERACT_AT` — рука + точка взаимодействия (local coords, float).
    InteractionAtLocation { hand: InteractionHand, location: Vec3 },
}

impl InteractAction {
    /// `getType`.
    pub fn action_type(&self) -> InteractActionType {
        match self {
            InteractAction::Interaction(_) => InteractActionType::Interact,
            InteractAction::Attack => InteractActionType::Attack,
            InteractAction::InteractionAtLocation { .. } => InteractActionType::InteractAt,
        }
    }
}

/// `ServerboundInteractPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServerboundInteractPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `action`.
    pub action: InteractAction,
    /// `usingSecondaryAction` (offhand).
    pub using_secondary_action: bool,
}

impl ServerboundInteractPacket {
    /// `createAttackPacket`.
    pub fn create_attack_packet(entity_id: i32, using_secondary_action: bool) -> Self {
        Self {
            entity_id,
            action: InteractAction::Attack,
            using_secondary_action,
        }
    }

    /// `createInteractionPacket(hand)`.
    pub fn create_interaction_packet(
        entity_id: i32,
        using_secondary_action: bool,
        hand: InteractionHand,
    ) -> Self {
        Self {
            entity_id,
            action: InteractAction::Interaction(hand),
            using_secondary_action,
        }
    }

    /// `createInteractionPacket(hand, location)`.
    pub fn create_interaction_packet_at(
        entity_id: i32,
        using_secondary_action: bool,
        hand: InteractionHand,
        interaction_location: Vec3,
    ) -> Self {
        Self {
            entity_id,
            action: InteractAction::InteractionAtLocation {
                hand,
                location: interaction_location,
            },
            using_secondary_action,
        }
    }

    /// `getEntityId`.
    pub fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    /// `isUsingSecondaryAction`.
    pub fn is_using_secondary_action(&self) -> bool {
        self.using_secondary_action
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundInteractPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundInteractPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let action_type = InteractActionType::by_id(buf.read_var_int());
                let action = match action_type {
                    InteractActionType::Interact => {
                        InteractAction::Interaction(InteractionHand::by_id(buf.read_var_int()))
                    }
                    InteractActionType::Attack => InteractAction::Attack,
                    InteractActionType::InteractAt => {
                        let location = Vec3::new(
                            buf.read_float() as f64,
                            buf.read_float() as f64,
                            buf.read_float() as f64,
                        );
                        let hand = InteractionHand::by_id(buf.read_var_int());
                        InteractAction::InteractionAtLocation { hand, location }
                    }
                };
                let using_secondary_action = buf.read_bool();
                ServerboundInteractPacket {
                    entity_id,
                    action,
                    using_secondary_action,
                }
            },
        )
    }
}

impl Packet for ServerboundInteractPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_interact::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        buf.write_var_int(self.action.action_type().ordinal());
        match self.action {
            InteractAction::Interaction(hand) => {
                buf.write_var_int(hand.ordinal());
            }
            InteractAction::Attack => {}
            InteractAction::InteractionAtLocation { hand, location } => {
                buf.write_float(location.x as f32);
                buf.write_float(location.y as f32);
                buf.write_float(location.z as f32);
                buf.write_var_int(hand.ordinal());
            }
        }
        buf.write_bool(self.using_secondary_action);
    }
}
