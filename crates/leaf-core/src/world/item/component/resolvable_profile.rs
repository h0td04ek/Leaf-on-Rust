//! Перенос `net.minecraft.world.item.component.ResolvableProfile`
//! — профиль на skull-итемах: either(GameProfile, Partial) + skinPatch.

use crate::authlib::game_profile::GameProfile;
use crate::authlib::property_map::PropertyMap;
use crate::entity::player::player_skin::PlayerSkinPatch;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::util::either::Either;
use crate::util::uuid::Uuid;

/// `ResolvableProfile.Partial` — (name?, id?, properties).
/// `Partial.createProfile` строит GameProfile через
/// `ResolvableProfile.createPartialProfile` (uuid = id или
/// offline-uuid от name, или NIL_UUID).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvableProfilePartial {
    /// `name`.
    pub name: Option<String>,
    /// `id`.
    pub id: Option<Uuid>,
    /// `properties`.
    pub properties: PropertyMap,
}

impl ResolvableProfilePartial {
    /// `new`.
    pub fn new(name: Option<String>, id: Option<Uuid>, properties: PropertyMap) -> Self {
        Self { name, id, properties }
    }

    /// `createProfile` — `createPartialProfile`.
    pub fn create_profile(&self) -> GameProfile {
        let name = self.name.clone().unwrap_or_default();
        let uuid = self
            .id
            .or_else(|| {
                if name.is_empty() {
                    None
                } else {
                    Some(Uuid::create_offline_player_uuid(&name))
                }
            })
            .unwrap_or_else(Uuid::nil);
        GameProfile::with_properties(uuid, name, self.properties.clone())
    }

    /// `STREAM_CODEC` — optional(PLAYER_NAME) + optional(UUID) +
    /// GAME_PROFILE_PROPERTIES.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        let name_enc = byte_buf_codecs::optional(byte_buf_codecs::PLAYER_NAME);
        let name_dec = byte_buf_codecs::optional(byte_buf_codecs::PLAYER_NAME);
        let uuid_codec = || {
            sync_fn(
                |buf: &mut FriendlyByteBuf, v: &Uuid| { buf.write_uuid(v); },
                |buf: &mut FriendlyByteBuf| buf.read_uuid(),
            )
        };
        let id_enc = byte_buf_codecs::optional(uuid_codec());
        let id_dec = byte_buf_codecs::optional(uuid_codec());
        let props_enc = byte_buf_codecs::game_profile_properties();
        let props_dec = byte_buf_codecs::game_profile_properties();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Self| {
                name_enc.encode(buf, &v.name);
                id_enc.encode(buf, &v.id);
                props_enc.encode(buf, &v.properties);
            },
            move |buf: &mut FriendlyByteBuf| {
                let name = name_dec.decode(buf);
                let id = id_dec.decode(buf);
                let properties = props_dec.decode(buf);
                ResolvableProfilePartial { name, id, properties }
            },
        )
    }
}

/// `ResolvableProfile` — абстрактный в Java; на проводе это
/// `either(GAME_PROFILE, Partial)` + `PlayerSkin.Patch`.
/// Храним `Either<GameProfile, ResolvableProfilePartial>` + patch.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvableProfile {
    /// `unpack()` — either-содержимое.
    pub contents: Either<GameProfile, ResolvableProfilePartial>,
    /// `skinPatch`.
    pub skin_patch: PlayerSkinPatch,
}

impl ResolvableProfile {
    /// `create`.
    pub fn new(contents: Either<GameProfile, ResolvableProfilePartial>, skin_patch: PlayerSkinPatch) -> Self {
        Self { contents, skin_patch }
    }

    /// `createResolved(GameProfile)` — `Either.left(profile)` + EMPTY patch.
    pub fn create_resolved(profile: GameProfile) -> Self {
        Self { contents: Either::Left(profile), skin_patch: PlayerSkinPatch::empty() }
    }

    /// `partialProfile()` — частичный GameProfile.
    pub fn partial_profile(&self) -> GameProfile {
        match &self.contents {
            Either::Left(p) => p.clone(),
            Either::Right(p) => p.create_profile(),
        }
    }

    /// `name()` — optional имя.
    pub fn name(&self) -> Option<&str> {
        match &self.contents {
            Either::Left(p) => {
                if p.name.is_empty() { None } else { Some(p.name.as_str()) }
            }
            Either::Right(p) => p.name.as_deref(),
        }
    }

    /// `STREAM_CODEC` — either(GAME_PROFILE, Partial) + Patch.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ResolvableProfile> + Send + Sync + 'static {
        let contents_enc = byte_buf_codecs::either(
            byte_buf_codecs::game_profile(),
            ResolvableProfilePartial::stream_codec(),
        );
        let contents_dec = byte_buf_codecs::either(
            byte_buf_codecs::game_profile(),
            ResolvableProfilePartial::stream_codec(),
        );
        let patch_enc = PlayerSkinPatch::stream_codec();
        let patch_dec = PlayerSkinPatch::stream_codec();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &ResolvableProfile| {
                contents_enc.encode(buf, &v.contents);
                patch_enc.encode(buf, &v.skin_patch);
            },
            move |buf: &mut FriendlyByteBuf| {
                let contents = contents_dec.decode(buf);
                let skin_patch = patch_dec.decode(buf);
                ResolvableProfile { contents, skin_patch }
            },
        )
    }
}
