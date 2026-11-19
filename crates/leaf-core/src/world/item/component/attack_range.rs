//! Перенос `net.minecraft.world.item.component.AttackRange`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `AttackRange` — 6 float-полей.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttackRange {
    /// `minRange`.
    pub min_range: f32,
    /// `maxRange`.
    pub max_range: f32,
    /// `minCreativeRange`.
    pub min_creative_range: f32,
    /// `maxCreativeRange`.
    pub max_creative_range: f32,
    /// `hitboxMargin`.
    pub hitbox_margin: f32,
    /// `mobFactor`.
    pub mob_factor: f32,
}

impl AttackRange {
    /// `CODEC_DEFAULT` (Paper defaults).
    pub const DEFAULT: AttackRange = AttackRange {
        min_range: 0.0,
        max_range: 3.0,
        min_creative_range: 0.0,
        max_creative_range: 5.0,
        hitbox_margin: 0.3,
        mob_factor: 1.0,
    };

    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        min_range: f32,
        max_range: f32,
        min_creative_range: f32,
        max_creative_range: f32,
        hitbox_margin: f32,
        mob_factor: f32,
    ) -> Self {
        Self { min_range, max_range, min_creative_range, max_creative_range, hitbox_margin, mob_factor }
    }

    /// `AttackRange.defaultFor(livingEntity)` — диапазоны из
    /// атрибута `ENTITY_INTERACTION_RANGE`.
    pub fn default_for(
        living: &crate::world::entity::living_entity::LivingEntity,
    ) -> Self {
        let range = living.get_attribute_value_by_id_ro(
            &crate::entity::ai::attributes::attributes::entity_interaction_range(),
        ) as f32;
        Self {
            min_range: 0.0,
            max_range: range,
            min_creative_range: 0.0,
            max_creative_range: range,
            hitbox_margin: 0.0,
            mob_factor: 1.0,
        }
    }

    /// `effectiveMinRange(entity)` — spectator → 0; player →
    /// creative-мин или обычный; иначе `minRange * mobFactor`.
    pub fn effective_min_range(&self, entity: &crate::world::entity::entity::Entity) -> f32 {
        if entity.is_spectator() {
            0.0
        } else if entity.is_creative() {
            self.min_creative_range
        } else if entity.is_mob {
            self.min_range * self.mob_factor
        } else {
            self.min_range
        }
    }

    /// `effectiveMaxRange(entity)`.
    pub fn effective_max_range(&self, entity: &crate::world::entity::entity::Entity) -> f32 {
        if entity.is_creative() {
            self.max_creative_range
        } else if entity.is_mob {
            self.max_range * self.mob_factor
        } else {
            self.max_range
        }
    }

    /// `isInRange(living, location)` — расстояние от глаз до точки
    /// в пределах `[min, max]` эффективного диапазона.
    pub fn is_in_range(
        &self,
        living: &crate::world::entity::living_entity::LivingEntity,
        location: &crate::world::phys::Vec3,
    ) -> bool {
        self.is_in_range_with(living, location, |a, b| a.distance_to_sqr(b), 0.0)
    }

    /// `isInRange(living, distanceToSqrFunction, offset)`.
    pub fn is_in_range_with(
        &self,
        living: &crate::world::entity::living_entity::LivingEntity,
        location: &crate::world::phys::Vec3,
        distance_fn: impl Fn(&crate::world::phys::Vec3, &crate::world::phys::Vec3) -> f64,
        offset: f64,
    ) -> bool {
        let eye = living.get_eye_position();
        let min = self.effective_min_range(&living.entity) as f64;
        let max = self.effective_max_range(&living.entity) as f64;
        let d = distance_fn(&eye, location) + offset;
        let margin = self.hitbox_margin as f64;
        d <= (max + margin) * (max + margin) && d >= (min - margin) * (min - margin)
    }

    /// `STREAM_CODEC` — 6× FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AttackRange> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &AttackRange| {
                buf.write_float(v.min_range);
                buf.write_float(v.max_range);
                buf.write_float(v.min_creative_range);
                buf.write_float(v.max_creative_range);
                buf.write_float(v.hitbox_margin);
                buf.write_float(v.mob_factor);
            },
            |buf: &mut FriendlyByteBuf| {
                let min_range = buf.read_float();
                let max_range = buf.read_float();
                let min_creative_range = buf.read_float();
                let max_creative_range = buf.read_float();
                let hitbox_margin = buf.read_float();
                let mob_factor = buf.read_float();
                AttackRange::new(
                    min_range,
                    max_range,
                    min_creative_range,
                    max_creative_range,
                    hitbox_margin,
                    mob_factor,
                )
            },
        )
    }
}
