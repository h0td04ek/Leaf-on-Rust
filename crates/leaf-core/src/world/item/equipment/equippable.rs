//! Перенос `net.minecraft.world.item.equipment.Equippable`.

use crate::core::holder::Holder;
use crate::core::holder_set::{holder_set_stream_codec, HolderSet};
use crate::entity::entity_type::EntityType;
use crate::entity::equipment_slot::EquipmentSlot;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::sounds::sound_event::SoundEvent;
use std::sync::Arc;

/// `EquipmentAsset`-ключ (`ResourceKey<EquipmentAsset>` на проводе —
/// `Identifier`; registry `equipment_asset`).
pub type AssetId = ResourceKey<crate::world::item::equipment::equipment_asset::EquipmentAsset>;

/// `Equippable` — параметры экипировки предмета.
#[derive(Clone, Debug, PartialEq)]
pub struct Equippable {
    /// `slot`.
    pub slot: EquipmentSlot,
    /// `equipSound`.
    pub equip_sound: Holder<SoundEvent>,
    /// `assetId`.
    pub asset_id: Option<AssetId>,
    /// `cameraOverlay`.
    pub camera_overlay: Option<Identifier>,
    /// `allowedEntities`.
    pub allowed_entities: Option<HolderSet<EntityType>>,
    /// `dispensable`.
    pub dispensable: bool,
    /// `swappable`.
    pub swappable: bool,
    /// `damageOnHurt`.
    pub damage_on_hurt: bool,
    /// `equipOnInteract`.
    pub equip_on_interact: bool,
    /// `canBeSheared`.
    pub can_be_sheared: bool,
    /// `shearingSound`.
    pub shearing_sound: Holder<SoundEvent>,
}

impl Equippable {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        slot: EquipmentSlot,
        equip_sound: Holder<SoundEvent>,
        asset_id: Option<AssetId>,
        camera_overlay: Option<Identifier>,
        allowed_entities: Option<HolderSet<EntityType>>,
        dispensable: bool,
        swappable: bool,
        damage_on_hurt: bool,
        equip_on_interact: bool,
        can_be_sheared: bool,
        shearing_sound: Holder<SoundEvent>,
    ) -> Self {
        Self {
            slot,
            equip_sound,
            asset_id,
            camera_overlay,
            allowed_entities,
            dispensable,
            swappable,
            damage_on_hurt,
            equip_on_interact,
            can_be_sheared,
            shearing_sound,
        }
    }

    /// `Equippable.STREAM_CODEC` — 11 полей по порядку Java.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Equippable> + Send + Sync + 'static {
        // assetId: optional(ResourceKey.streamCodec(ROOT_ID)) — на
        // проводе optional(Identifier).
        let asset_opt_enc = byte_buf_codecs::optional(byte_buf_codecs::IDENTIFIER);
        let asset_opt_dec = byte_buf_codecs::optional(byte_buf_codecs::IDENTIFIER);
        let overlay_opt_enc = byte_buf_codecs::optional(byte_buf_codecs::IDENTIFIER);
        let overlay_opt_dec = byte_buf_codecs::optional(byte_buf_codecs::IDENTIFIER);
        let entities_enc = byte_buf_codecs::optional(holder_set_stream_codec::<EntityType>());
        let entities_dec = byte_buf_codecs::optional(holder_set_stream_codec::<EntityType>());
        let _ = Arc::new(()); // keep Arc import used pattern consistent
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Equippable| {
                EquipmentSlot::stream_codec().encode(buf, &v.slot);
                SoundEvent::stream_codec().encode(buf, &v.equip_sound);
                asset_opt_enc.encode(
                    buf,
                    &v.asset_id.as_ref().map(|k| k.identifier.clone()),
                );
                overlay_opt_enc.encode(buf, &v.camera_overlay);
                entities_enc.encode(buf, &v.allowed_entities);
                buf.write_bool(v.dispensable);
                buf.write_bool(v.swappable);
                buf.write_bool(v.damage_on_hurt);
                buf.write_bool(v.equip_on_interact);
                buf.write_bool(v.can_be_sheared);
                SoundEvent::stream_codec().encode(buf, &v.shearing_sound);
            },
            move |buf: &mut FriendlyByteBuf| {
                let slot = EquipmentSlot::stream_codec().decode(buf);
                let equip_sound = SoundEvent::stream_codec().decode(buf);
                let asset_id = asset_opt_dec
                    .decode(buf)
                    .map(|id: Identifier| {
                        ResourceKey::create_with_registry_name(
                            Identifier::parse("minecraft:equipment_asset").expect("equipment_asset registry"),
                            id,
                        )
                    });
                let camera_overlay = overlay_opt_dec.decode(buf);
                let allowed_entities = entities_dec.decode(buf);
                let dispensable = buf.read_bool();
                let swappable = buf.read_bool();
                let damage_on_hurt = buf.read_bool();
                let equip_on_interact = buf.read_bool();
                let can_be_sheared = buf.read_bool();
                let shearing_sound = SoundEvent::stream_codec().decode(buf);
                Equippable::new(
                    slot,
                    equip_sound,
                    asset_id,
                    camera_overlay,
                    allowed_entities,
                    dispensable,
                    swappable,
                    damage_on_hurt,
                    equip_on_interact,
                    can_be_sheared,
                    shearing_sound,
                )
            },
        )
    }
}
