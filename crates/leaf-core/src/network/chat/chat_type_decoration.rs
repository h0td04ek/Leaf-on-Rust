//! Перенос `net.minecraft.network.chat.ChatTypeDecoration` (+ `Parameter`).

use crate::network::chat::common_components;
use crate::network::chat::component::Component;
use crate::network::chat::style::Style;
use crate::network::chat::style::style_stream_codec;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ChatTypeDecoration.Parameter` — селектор параметра декорации.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatTypeDecorationParameter {
    /// `SENDER` (id 0).
    Sender = 0,
    /// `TARGET` (id 1).
    Target = 1,
    /// `CONTENT` (id 2).
    Content = 2,
}

impl ChatTypeDecorationParameter {
    /// `values()`-порядок.
    pub const VALUES: [ChatTypeDecorationParameter; 3] = [
        ChatTypeDecorationParameter::Sender,
        ChatTypeDecorationParameter::Target,
        ChatTypeDecorationParameter::Content,
    ];

    /// `byId` — id (ByteBufCodecs.idMapper, OutOfBounds→ZERO).
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(3) as usize]
    }

    /// `id`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `getSerializedName`.
    pub fn serialized_name(&self) -> &'static str {
        match self {
            ChatTypeDecorationParameter::Sender => "sender",
            ChatTypeDecorationParameter::Target => "target",
            ChatTypeDecorationParameter::Content => "content",
        }
    }
}

/// `ChatTypeDecoration` — translationKey + parameters + style.
#[derive(Clone, Debug, PartialEq)]
pub struct ChatTypeDecoration {
    /// `translationKey`.
    pub translation_key: String,
    /// `parameters`.
    pub parameters: Vec<ChatTypeDecorationParameter>,
    /// `style`.
    pub style: Style,
}

impl ChatTypeDecoration {
    pub fn new(
        translation_key: String,
        parameters: Vec<ChatTypeDecorationParameter>,
        style: Style,
    ) -> Self {
        Self { translation_key, parameters, style }
    }

    /// `withSender(translationKey)`.
    pub fn with_sender(translation_key: &str) -> Self {
        Self::new(
            translation_key.to_string(),
            vec![
                ChatTypeDecorationParameter::Sender,
                ChatTypeDecorationParameter::Content,
            ],
            Style::empty(),
        )
    }

    /// `incomingDirectMessage(translationKey)`.
    pub fn incoming_direct_message(translation_key: &str) -> Self {
        // Style.EMPTY.withColor(GRAY).withItalic(true)
        let mut style = Style::empty();
        style.color = crate::network::chat::text_color::TextColor::from_legacy_format(
            crate::network::chat::chat_formatting::ChatFormatting::Gray,
        );
        style.italic = Some(true);
        Self::new(
            translation_key.to_string(),
            vec![
                ChatTypeDecorationParameter::Sender,
                ChatTypeDecorationParameter::Content,
            ],
            style,
        )
    }

    /// `outgoingDirectMessage(translationKey)`.
    pub fn outgoing_direct_message(translation_key: &str) -> Self {
        let mut style = Style::empty();
        style.color = crate::network::chat::text_color::TextColor::from_legacy_format(
            crate::network::chat::chat_formatting::ChatFormatting::Gray,
        );
        style.italic = Some(true);
        Self::new(
            translation_key.to_string(),
            vec![
                ChatTypeDecorationParameter::Target,
                ChatTypeDecorationParameter::Content,
            ],
            style,
        )
    }

    /// `teamMessage(translationKey)`.
    pub fn team_message(translation_key: &str) -> Self {
        Self::new(
            translation_key.to_string(),
            vec![
                ChatTypeDecorationParameter::Target,
                ChatTypeDecorationParameter::Sender,
                ChatTypeDecorationParameter::Content,
            ],
            Style::empty(),
        )
    }

    /// `decorate(content, boundChatType)` — Component.translatable + style.
    /// Точная реализация translatable-декорации — через `Component::translatable`
    /// когда портируется MutableComponent; здесь возвращаем выборку параметров.
    pub fn resolve_parameters(
        &self,
        content: &Component,
        bound_chat_type: &crate::network::chat::chat_type::ChatTypeBound,
    ) -> Vec<Component> {
        self.parameters
            .iter()
            .map(|p| match p {
                ChatTypeDecorationParameter::Sender => bound_chat_type.name.clone(),
                ChatTypeDecorationParameter::Target => bound_chat_type
                    .target_name
                    .clone()
                    .unwrap_or_else(|| common_components::empty()),
                ChatTypeDecorationParameter::Content => content.clone(),
            })
            .collect()
    }

    /// `STREAM_CODEC` — STRING_UTF8 + list(Parameter idMapper) + Style.
    pub fn stream_codec()
    -> impl StreamCodec<FriendlyByteBuf, ChatTypeDecoration> {
        crate::network::codec::stream_codec::FnCodec::new(
            |buf: &mut FriendlyByteBuf, d: &ChatTypeDecoration| {
                buf.write_utf(&d.translation_key).expect("translation key too long");
                buf.write_collection(&d.parameters, |b, p| { b.write_var_int(p.id()); });
                style_stream_codec().encode(buf, &d.style);
            },
            |buf: &mut FriendlyByteBuf| {
                let translation_key = buf.read_utf().expect("invalid translation key");
                let parameters = buf
                    .read_collection(|_| Vec::new(), |b| {
                        ChatTypeDecorationParameter::by_id(b.read_var_int())
                    })
                    .into_iter()
                    .collect::<Vec<_>>();
                let parameters = parameters;
                let style = style_stream_codec().decode(buf);
                ChatTypeDecoration::new(translation_key, parameters, style)
            },
        )
    }
}
