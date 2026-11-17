//! Перенос `net.minecraft.world.item.component.ProvidesTrimMaterial`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::either_holder::EitherHolder;
use crate::world::item::equipment::trim::trim_material::TrimMaterial;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;

/// `ProvidesTrimMaterial` — `EitherHolder<TrimMaterial>`.
#[derive(Clone, Debug, PartialEq)]
pub struct ProvidesTrimMaterial {
    /// `material`.
    pub material: EitherHolder<TrimMaterial>,
}

impl ProvidesTrimMaterial {
    /// `new`.
    pub fn new(material: EitherHolder<TrimMaterial>) -> Self {
        Self { material }
    }

    /// `of_key`.
    pub fn of_key(key: ResourceKey<TrimMaterial>) -> Self {
        Self { material: EitherHolder::of_key(key) }
    }

    /// `STREAM_CODEC` — `EitherHolder.streamCodec(TRIM_MATERIAL,
    /// TrimMaterial.DIRECT_STREAM_CODEC)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ProvidesTrimMaterial> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &ProvidesTrimMaterial| {
                EitherHolder::stream_codec(
                    Identifier::minecraft("trim_material"),
                    TrimMaterial::stream_codec(),
                )
                .encode(buf, &v.material);
            },
            |buf: &mut FriendlyByteBuf| {
                let material = EitherHolder::stream_codec(
                    Identifier::minecraft("trim_material"),
                    TrimMaterial::stream_codec(),
                )
                .decode(buf);
                ProvidesTrimMaterial::new(material)
            },
        )
    }
}
