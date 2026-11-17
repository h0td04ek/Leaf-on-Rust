//! Перенос `net.minecraft.world.entity.player.PlayerSkin.Patch` —
//! опциональные переопределения скина (texture/cape/elytra/model).
//!
//! `ClientAsset.ResourceTexture` на проводе — `Identifier`
//! (один; texturePath выводится из id).

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::entity::player::player_model_type::PlayerModelType;

/// `ClientAsset.ResourceTexture` — (id, texturePath); на проводе
/// только `id` (`Identifier.STREAM_CODEC.map`).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceTexture {
    /// `id` — asset id.
    pub id: Identifier,
}

impl ResourceTexture {
    /// `new ResourceTexture(id)`.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }

    /// `texturePath` — в Java производный от id
    /// (`textures/entity/...`); здесь — id (полный порт asset-путей
    /// — с client-стадией).
    pub fn texture_path(&self) -> &Identifier {
        &self.id
    }

    /// `STREAM_CODEC` — `Identifier.STREAM_CODEC.map`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ResourceTexture> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &ResourceTexture| {
                buf.write_identifier(&v.id).expect("resource texture id");
            },
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_identifier().expect("resource texture id");
                ResourceTexture { id }
            },
        )
    }
}

/// `PlayerSkin.Patch` — 4 optional поля.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerSkinPatch {
    /// `body`/`texture`.
    pub texture: Option<ResourceTexture>,
    /// `cape`.
    pub cape: Option<ResourceTexture>,
    /// `elytra`.
    pub elytra: Option<ResourceTexture>,
    /// `model`.
    pub model: Option<PlayerModelType>,
}

impl PlayerSkinPatch {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `STREAM_CODEC` — optional(ResourceTexture)×3 + optional(PlayerModelType).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PlayerSkinPatch> + Send + Sync + 'static {
        let tex_enc1 = byte_buf_codecs::optional(ResourceTexture::stream_codec());
        let tex_enc2 = byte_buf_codecs::optional(ResourceTexture::stream_codec());
        let tex_enc3 = byte_buf_codecs::optional(ResourceTexture::stream_codec());
        let tex_dec1 = byte_buf_codecs::optional(ResourceTexture::stream_codec());
        let tex_dec2 = byte_buf_codecs::optional(ResourceTexture::stream_codec());
        let tex_dec3 = byte_buf_codecs::optional(ResourceTexture::stream_codec());
        let model_enc = byte_buf_codecs::optional(PlayerModelType::stream_codec());
        let model_dec = byte_buf_codecs::optional(PlayerModelType::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &PlayerSkinPatch| {
                tex_enc1.encode(buf, &v.texture);
                tex_enc2.encode(buf, &v.cape);
                tex_enc3.encode(buf, &v.elytra);
                model_enc.encode(buf, &v.model);
            },
            move |buf: &mut FriendlyByteBuf| PlayerSkinPatch {
                texture: tex_dec1.decode(buf),
                cape: tex_dec2.decode(buf),
                elytra: tex_dec3.decode(buf),
                model: model_dec.decode(buf),
            },
        )
    }
}
