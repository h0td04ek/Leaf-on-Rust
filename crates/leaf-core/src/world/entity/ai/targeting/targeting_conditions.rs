//! Перенос `net.minecraft.world.entity.ai.targeting.TargetingConditions`.

use crate::world::difficulty::Difficulty;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::entity::mob::Mob;
use crate::world::level::level_reader::LevelReader;
use std::sync::Arc;

/// `MIN_VISIBILITY_DISTANCE_FOR_INVISIBLE_TARGET`.
const MIN_VISIBILITY_DISTANCE_FOR_INVISIBLE_TARGET: f64 = 2.0;

/// `TargetingConditions.Selector` — `test(entity, level)`.
pub type Selector =
    Arc<dyn Fn(&LivingEntity, &dyn LevelReader) -> bool + Send + Sync>;

/// `TargetingConditions` — условия валидности цели:
/// combat/non-combat, range (с учётом невидимости),
/// line-of-sight, тестирование невидимости, selector.
#[derive(Clone)]
pub struct TargetingConditions {
    /// `isCombat`.
    is_combat: bool,
    /// `range` (-1 = без ограничения).
    range: f64,
    /// `checkLineOfSight`.
    check_line_of_sight: bool,
    /// `testInvisible`.
    test_invisible: bool,
    /// `selector`.
    selector: Option<Selector>,
}

impl TargetingConditions {
    /// `forCombat()`.
    pub fn for_combat() -> Self {
        Self {
            is_combat: true,
            range: -1.0,
            check_line_of_sight: true,
            test_invisible: true,
            selector: None,
        }
    }

    /// `forNonCombat()`.
    pub fn for_non_combat() -> Self {
        Self {
            is_combat: false,
            range: -1.0,
            check_line_of_sight: true,
            test_invisible: true,
            selector: None,
        }
    }

    /// `DEFAULT` — `forCombat()`.
    pub fn default_conditions() -> Self {
        Self::for_combat()
    }

    /// `copy()`.
    pub fn copy(&self) -> Self {
        Self {
            is_combat: self.is_combat,
            range: self.range,
            check_line_of_sight: self.check_line_of_sight,
            test_invisible: self.test_invisible,
            selector: self.selector.clone(),
        }
    }

    /// `range(range)`.
    pub fn range(&mut self, range: f64) -> &mut Self {
        self.range = range;
        self
    }

    /// Текущее значение range (для сканов снаружи).
    pub fn range_value(&self) -> f64 {
        self.range
    }

    /// `ignoreLineOfSight()`.
    pub fn ignore_line_of_sight(&mut self) -> &mut Self {
        self.check_line_of_sight = false;
        self
    }

    /// `ignoreInvisibilityTesting()`.
    pub fn ignore_invisibility_testing(&mut self) -> &mut Self {
        self.test_invisible = false;
        self
    }

    /// `selector(selector)`.
    pub fn selector(&mut self, selector: Selector) -> &mut Self {
        self.selector = Some(selector);
        self
    }

    /// `test(level, entity, target)` — `entity` = атакующий
    /// (`None` = безатакующий контекст). `mob` используется для
    /// `canAttack`/`canAttackType`/`isAlliedTo` и line-of-sight
    /// (Sensing — &mut, поэтому берём `&mut Mob`).
    pub fn test(
        &self,
        level: &Arc<dyn LevelReader>,
        mob: Option<&mut Mob>,
        target: &LivingEntity,
    ) -> bool {
        // entity == target проверка делается снаружи (нет self-id
        // у LivingEntity-view; uuid-сравнение — на вызывающей стороне).
        if !target.can_be_seen_by_anyone() {
            return false;
        }
        let level_ref: &dyn LevelReader = level.as_ref();
        if let Some(sel) = &self.selector {
            if !sel(target, level_ref) {
                return false;
            }
        }

        if let Some(m) = mob.as_deref() {
            // combat-ветка с entity != null
            if self.is_combat {
                if !m.living.can_attack(target)
                    || !m.living.can_attack_type(&target.entity.entity_type)
                    || m.living.is_allied_to(&target.entity)
                {
                    return false;
                }
            }

            if self.range > 0.0 {
                let follow_range_raw = self.range;
                let d1 = m.living.distance_to_sqr(&target.entity);
                if d1 > follow_range_raw * follow_range_raw {
                    return false;
                }
                let d = if self.test_invisible {
                    target.get_visibility_percent(&m.living.entity)
                } else {
                    1.0
                };
                let max = (follow_range_raw * d).max(MIN_VISIBILITY_DISTANCE_FOR_INVISIBLE_TARGET);
                if d1 > max * max {
                    return false;
                }
            }
        } else {
            // entity == null combat-ветка
            if self.is_combat
                && (!target.can_be_seen_as_enemy()
                    || level.difficulty() == Difficulty::Peaceful)
            {
                return false;
            }
        }

        // checkLineOfSight — только для Mob-атакующего.
        if self.check_line_of_sight {
            if let Some(m) = mob {
                if !m.has_line_of_sight(&target.entity) {
                    return false;
                }
            }
        }
        true
    }
}
