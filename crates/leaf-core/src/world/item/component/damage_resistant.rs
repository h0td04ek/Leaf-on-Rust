//! Перенос `net.minecraft.world.item.component.DamageResistant`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::tags::TagKey;
use crate::world::damagesource::damage_type::DamageType;

/// `DamageResistant` — `TagKey<DamageType>` типов урона,
/// к которым предмет сопротивляется (стойкость брони).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageResistant {
    /// `types`.
    pub types: TagKey<DamageType>,
}

impl DamageResistant {
    /// `STREAM_CODEC` — `TagKey.streamCodec(DAMAGE_TYPE)`.
    /// Registry-ключ damage_type нужен для типа TagKey — см.
    /// `Registries.DAMAGE_TYPE` (порт реестров — отдельный этап);
    /// здесь хранится как часть значения.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, DamageResistant> + Send + Sync + 'static {
        // В Java codec зависит от ResourceKey реестра; значение TagKey
        // уже несёт registry-ключ, а на проводе пишется только location.
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &DamageResistant| {
                buf.write_identifier(v.types.location()).expect("tag location");
            },
            |buf: &mut FriendlyByteBuf| {
                let location = buf.read_identifier().expect("tag location");
                // registry-ключ восстанавливается из контекста —
                // DAMAGE_TYPE. Создаём через placeholder registry key:
                // ResourceKey реестра портируется в core::registry.
                let registry = crate::resources::resource_key::ResourceKey::create_registry_key(
                    crate::resources::identifier::Identifier::parse("damage_type").unwrap(),
                );
                DamageResistant {
                    types: TagKey::create(registry, location),
                }
            },
        )
    }
}
