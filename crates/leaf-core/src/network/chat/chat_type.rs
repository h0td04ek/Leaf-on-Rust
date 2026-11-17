//! Перенос `net.minecraft.network.chat.ChatType` (+ `Bound`).

use crate::core::holder::{Holder, HolderStreamCodec};
use crate::network::chat::chat_type_decoration::{
    ChatTypeDecoration, ChatTypeDecorationParameter,
};
use crate::network::chat::component::Component;
use crate::network::chat::component_json::{
    component_stream_codec, optional_component_stream_codec,
};
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::{Identifier, ResourceKey};

/// `ChatType` — chat + narration декорации.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatType {
    /// `chat`.
    pub chat: ChatTypeDecoration,
    /// `narration`.
    pub narration: ChatTypeDecoration,
}

impl ChatType {
    /// `new ChatType(chat, narration)`.
    pub fn new(chat: ChatTypeDecoration, narration: ChatTypeDecoration) -> Self {
        Self { chat, narration }
    }

    /// `DIRECT_STREAM_CODEC`.
    pub fn direct_stream_codec() -> impl StreamCodec<FriendlyByteBuf, ChatType> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, t: &ChatType| {
                ChatTypeDecoration::stream_codec().encode(buf, &t.chat);
                ChatTypeDecoration::stream_codec().encode(buf, &t.narration);
            },
            |buf: &mut FriendlyByteBuf| {
                let chat = ChatTypeDecoration::stream_codec().decode(buf);
                let narration = ChatTypeDecoration::stream_codec().decode(buf);
                ChatType::new(chat, narration)
            },
        )
    }
}

/// `ChatType.Bound` — holder + name + optional targetName.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatTypeBound {
    /// `chatType` — `Holder<ChatType>`.
    pub chat_type: Holder<ChatType>,
    /// `name`.
    pub name: Component,
    /// `targetName`.
    pub target_name: Option<Component>,
}

impl ChatTypeBound {
    /// `new Bound(chatType, name)` — без target.
    pub fn new(chat_type: Holder<ChatType>, name: Component) -> Self {
        Self { chat_type, name, target_name: None }
    }

    /// `new Bound(chatType, name, targetName)`.
    pub fn with_target(
        chat_type: Holder<ChatType>,
        name: Component,
        target_name: Option<Component>,
    ) -> Self {
        Self { chat_type, name, target_name }
    }

    /// `withTargetName`.
    pub fn with_target_name(&self, target_name: Component) -> Self {
        Self {
            chat_type: self.chat_type.clone(),
            name: self.name.clone(),
            target_name: Some(target_name),
        }
    }

    /// `decorate(content)`.
    pub fn decorate(&self, content: &Component) -> Component {
        match &self.chat_type {
            Holder::Direct(t) => {
                let params: Vec<crate::network::chat::component_contents::TranslatableArg> = t
                    .chat
                    .resolve_parameters(content, self)
                    .into_iter()
                    .map(|c| crate::network::chat::component_contents::TranslatableArg::Component(Box::new(c)))
                    .collect();
                let mut c = Component::translatable(t.chat.translation_key.clone(), params);
                c.with_style(t.chat.style.clone());
                c
            }
            Holder::Reference(id) => {
                // Реестр ChatType ещё не портирован — вернуть content с именем.
                let _ = id;
                content.clone()
            }
        }
    }

    /// `STREAM_CODEC` — holder + TRUSTED component + optional TRUSTED component.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ChatTypeBound> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, b: &ChatTypeBound| {
                let holder_codec = HolderStreamCodec::new(ChatType::direct_stream_codec());
                holder_codec.encode(buf, &b.chat_type);
                component_stream_codec().encode(buf, &b.name);
                optional_component_stream_codec().encode(buf, &b.target_name);
            },
            |buf: &mut FriendlyByteBuf| {
                let holder_codec = HolderStreamCodec::new(ChatType::direct_stream_codec());
                let chat_type = holder_codec.decode(buf);
                let name = component_stream_codec().decode(buf);
                let target_name = optional_component_stream_codec().decode(buf);
                ChatTypeBound { chat_type, name, target_name }
            },
        )
    }
}

/// `ChatType.CHAT` — ResourceKey `minecraft:chat`.
pub fn chat_key() -> ResourceKey<ChatType> {
    ResourceKey::create_with_registry_name(
        crate::core::registry::registries::chat_type(),
        Identifier::minecraft("chat"),
    )
}

/// Прочие стандартные ключи `ChatType`.
pub mod keys {
    use super::*;
    fn key(name: &str) -> ResourceKey<ChatType> {
        ResourceKey::create_with_registry_name(crate::core::registry::registries::chat_type(), Identifier::minecraft(name))
    }
    /// `SAY_COMMAND`.
    pub fn say_command() -> ResourceKey<ChatType> { key("say_command") }
    /// `MSG_COMMAND_INCOMING`.
    pub fn msg_command_incoming() -> ResourceKey<ChatType> { key("msg_command_incoming") }
    /// `MSG_COMMAND_OUTGOING`.
    pub fn msg_command_outgoing() -> ResourceKey<ChatType> { key("msg_command_outgoing") }
    /// `TEAM_MSG_COMMAND_INCOMING`.
    pub fn team_msg_command_incoming() -> ResourceKey<ChatType> { key("team_msg_command_incoming") }
    /// `TEAM_MSG_COMMAND_OUTGOING`.
    pub fn team_msg_command_outgoing() -> ResourceKey<ChatType> { key("team_msg_command_outgoing") }
    /// `EMOTE_COMMAND`.
    pub fn emote_command() -> ResourceKey<ChatType> { key("emote_command") }
}


