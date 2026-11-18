//! Перенос `net.minecraft.advancements.DisplayInfo`.

use crate::advancements::advancement_type::AdvancementType;
use crate::network::chat::component::Component;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::world::item::item_stack::ItemStack;

/// `ClientAsset.ResourceTexture` — идентификатор текстуры.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceTexture {
    /// `id` — идентификатор ресурса.
    pub id: Identifier,
}

impl ResourceTexture {
    /// `new(id)`.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }
}

/// `DisplayInfo` — информация об отображении advancement.
#[derive(Clone, Debug)]
pub struct DisplayInfo {
    /// `title`.
    pub title: Component,
    /// `description`.
    pub description: Component,
    /// `icon`.
    pub icon: ItemStack,
    /// `background`.
    pub background: Option<ResourceTexture>,
    /// `type`.
    pub ty: AdvancementType,
    /// `showToast`.
    pub show_toast: bool,
    /// `announceChat`.
    pub announce_chat: bool,
    /// `hidden`.
    pub hidden: bool,
    /// `x` — позиция на дереве.
    pub x: f32,
    /// `y`.
    pub y: f32,
}

impl DisplayInfo {
    /// `new(...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        icon: ItemStack,
        title: Component,
        description: Component,
        background: Option<ResourceTexture>,
        ty: AdvancementType,
        show_toast: bool,
        announce_chat: bool,
        hidden: bool,
    ) -> Self {
        Self {
            title, description, icon, background, ty,
            show_toast, announce_chat, hidden,
            x: 0.0, y: 0.0,
        }
    }

    /// `setLocation`.
    pub fn set_location(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    /// `serializeToNetwork`.
    pub fn serialize_to_network(&self, buffer: &mut FriendlyByteBuf) {
        let codec = crate::network::chat::component_json::component_stream_codec();
        codec.encode(buffer, &self.title);
        codec.encode(buffer, &self.description);
        ItemStack::stream_codec().encode(buffer, &self.icon);
        buffer.write_var_int(self.ty as i32);
        let mut flags = 0i32;
        if self.background.is_some() { flags |= 1; }
        if self.show_toast { flags |= 2; }
        if self.hidden { flags |= 4; }
        buffer.write_int(flags);
        if let Some(bg) = &self.background {
            buffer.write_identifier(&bg.id);
        }
        buffer.write_float(self.x);
        buffer.write_float(self.y);
    }

    /// `fromNetwork`.
    pub fn from_network(buffer: &mut FriendlyByteBuf) -> Self {
        let codec = crate::network::chat::component_json::component_stream_codec();
        let title = codec.decode(buffer);
        let description = codec.decode(buffer);
        let icon = ItemStack::stream_codec().decode(buffer);
        let ty = AdvancementType::from_serialized_name(
            AdvancementType::VALUES[buffer.read_var_int() as usize].get_serialized_name()
        ).unwrap_or(AdvancementType::Task);
        let flags = buffer.read_int();
        let background = if (flags & 1) != 0 {
            Some(ResourceTexture::new(buffer.read_identifier().unwrap_or_else(|_| Identifier::from_namespace_and_path("minecraft", "unknown").unwrap_or_else(|_| Identifier::parse("minecraft:unknown").unwrap()))))
        } else {
            None
        };
        let show_toast = (flags & 2) != 0;
        let hidden = (flags & 4) != 0;
        let mut info = Self::new(icon, title, description, background, ty, show_toast, false, hidden);
        info.set_location(buffer.read_float(), buffer.read_float());
        info
    }
}
