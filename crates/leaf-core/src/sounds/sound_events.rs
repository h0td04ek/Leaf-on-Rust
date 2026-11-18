//! Перенос `net.minecraft.sounds.SoundEvents` — все реестровые
//! ключи звуковых событий 1.21.11 (1637+ записей).

use crate::core::holder::Holder;
use crate::core::registry::registries;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::sounds::sound_event::SoundEvent;

fn key(path: &str) -> ResourceKey<SoundEvent> {
    ResourceKey::create_with_registry_name(registries::sound_event(), Identifier::minecraft(path))
}

/// `SoundEvents.X` — Holder по registry-ключу.
/// `SoundEvents.ALLAY_AMBIENT_WITH_ITEM`.
pub fn allay_ambient_with_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.allay.ambient_with_item"))
}

/// `SoundEvents.ALLAY_AMBIENT_WITHOUT_ITEM`.
pub fn allay_ambient_without_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.allay.ambient_without_item"))
}

/// `SoundEvents.ALLAY_DEATH`.
pub fn allay_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.allay.death"))
}

/// `SoundEvents.ALLAY_HURT`.
pub fn allay_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.allay.hurt"))
}

/// `SoundEvents.ALLAY_ITEM_GIVEN`.
pub fn allay_item_given() -> Holder<SoundEvent> {
    Holder::Key(key("entity.allay.item_given"))
}

/// `SoundEvents.ALLAY_ITEM_TAKEN`.
pub fn allay_item_taken() -> Holder<SoundEvent> {
    Holder::Key(key("entity.allay.item_taken"))
}

/// `SoundEvents.ALLAY_THROW`.
pub fn allay_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.allay.item_thrown"))
}

/// `SoundEvents.AMBIENT_CAVE`.
pub fn ambient_cave() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.cave"))
}

/// `SoundEvents.AMBIENT_BASALT_DELTAS_ADDITIONS`.
pub fn ambient_basalt_deltas_additions() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.basalt_deltas.additions"))
}

/// `SoundEvents.AMBIENT_BASALT_DELTAS_LOOP`.
pub fn ambient_basalt_deltas_loop() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.basalt_deltas.loop"))
}

/// `SoundEvents.AMBIENT_BASALT_DELTAS_MOOD`.
pub fn ambient_basalt_deltas_mood() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.basalt_deltas.mood"))
}

/// `SoundEvents.AMBIENT_CRIMSON_FOREST_ADDITIONS`.
pub fn ambient_crimson_forest_additions() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.crimson_forest.additions"))
}

/// `SoundEvents.AMBIENT_CRIMSON_FOREST_LOOP`.
pub fn ambient_crimson_forest_loop() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.crimson_forest.loop"))
}

/// `SoundEvents.AMBIENT_CRIMSON_FOREST_MOOD`.
pub fn ambient_crimson_forest_mood() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.crimson_forest.mood"))
}

/// `SoundEvents.AMBIENT_NETHER_WASTES_ADDITIONS`.
pub fn ambient_nether_wastes_additions() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.nether_wastes.additions"))
}

/// `SoundEvents.AMBIENT_NETHER_WASTES_LOOP`.
pub fn ambient_nether_wastes_loop() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.nether_wastes.loop"))
}

/// `SoundEvents.AMBIENT_NETHER_WASTES_MOOD`.
pub fn ambient_nether_wastes_mood() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.nether_wastes.mood"))
}

/// `SoundEvents.AMBIENT_SOUL_SAND_VALLEY_ADDITIONS`.
pub fn ambient_soul_sand_valley_additions() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.soul_sand_valley.additions"))
}

/// `SoundEvents.AMBIENT_SOUL_SAND_VALLEY_LOOP`.
pub fn ambient_soul_sand_valley_loop() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.soul_sand_valley.loop"))
}

/// `SoundEvents.AMBIENT_SOUL_SAND_VALLEY_MOOD`.
pub fn ambient_soul_sand_valley_mood() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.soul_sand_valley.mood"))
}

/// `SoundEvents.AMBIENT_WARPED_FOREST_ADDITIONS`.
pub fn ambient_warped_forest_additions() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.warped_forest.additions"))
}

/// `SoundEvents.AMBIENT_WARPED_FOREST_LOOP`.
pub fn ambient_warped_forest_loop() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.warped_forest.loop"))
}

/// `SoundEvents.AMBIENT_WARPED_FOREST_MOOD`.
pub fn ambient_warped_forest_mood() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.warped_forest.mood"))
}

/// `SoundEvents.AMBIENT_UNDERWATER_ENTER`.
pub fn ambient_underwater_enter() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.underwater.enter"))
}

/// `SoundEvents.AMBIENT_UNDERWATER_EXIT`.
pub fn ambient_underwater_exit() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.underwater.exit"))
}

/// `SoundEvents.AMBIENT_UNDERWATER_LOOP`.
pub fn ambient_underwater_loop() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.underwater.loop"))
}

/// `SoundEvents.AMBIENT_UNDERWATER_LOOP_ADDITIONS`.
pub fn ambient_underwater_loop_additions() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.underwater.loop.additions"))
}

/// `SoundEvents.AMBIENT_UNDERWATER_LOOP_ADDITIONS_RARE`.
pub fn ambient_underwater_loop_additions_rare() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.underwater.loop.additions.rare"))
}

/// `SoundEvents.AMBIENT_UNDERWATER_LOOP_ADDITIONS_ULTRA_RARE`.
pub fn ambient_underwater_loop_additions_ultra_rare() -> Holder<SoundEvent> {
    Holder::Key(key("ambient.underwater.loop.additions.ultra_rare"))
}

/// `SoundEvents.AMETHYST_BLOCK_BREAK`.
pub fn amethyst_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_block.break"))
}

/// `SoundEvents.AMETHYST_BLOCK_CHIME`.
pub fn amethyst_block_chime() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_block.chime"))
}

/// `SoundEvents.AMETHYST_BLOCK_FALL`.
pub fn amethyst_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_block.fall"))
}

/// `SoundEvents.AMETHYST_BLOCK_HIT`.
pub fn amethyst_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_block.hit"))
}

/// `SoundEvents.AMETHYST_BLOCK_PLACE`.
pub fn amethyst_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_block.place"))
}

/// `SoundEvents.AMETHYST_BLOCK_RESONATE`.
pub fn amethyst_block_resonate() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_block.resonate"))
}

/// `SoundEvents.AMETHYST_BLOCK_STEP`.
pub fn amethyst_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_block.step"))
}

/// `SoundEvents.AMETHYST_CLUSTER_BREAK`.
pub fn amethyst_cluster_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_cluster.break"))
}

/// `SoundEvents.AMETHYST_CLUSTER_FALL`.
pub fn amethyst_cluster_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_cluster.fall"))
}

/// `SoundEvents.AMETHYST_CLUSTER_HIT`.
pub fn amethyst_cluster_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_cluster.hit"))
}

/// `SoundEvents.AMETHYST_CLUSTER_PLACE`.
pub fn amethyst_cluster_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_cluster.place"))
}

/// `SoundEvents.AMETHYST_CLUSTER_STEP`.
pub fn amethyst_cluster_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.amethyst_cluster.step"))
}

/// `SoundEvents.ANCIENT_DEBRIS_BREAK`.
pub fn ancient_debris_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.ancient_debris.break"))
}

/// `SoundEvents.ANCIENT_DEBRIS_STEP`.
pub fn ancient_debris_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.ancient_debris.step"))
}

/// `SoundEvents.ANCIENT_DEBRIS_PLACE`.
pub fn ancient_debris_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.ancient_debris.place"))
}

/// `SoundEvents.ANCIENT_DEBRIS_HIT`.
pub fn ancient_debris_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.ancient_debris.hit"))
}

/// `SoundEvents.ANCIENT_DEBRIS_FALL`.
pub fn ancient_debris_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.ancient_debris.fall"))
}

/// `SoundEvents.ANVIL_BREAK`.
pub fn anvil_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.anvil.break"))
}

/// `SoundEvents.ANVIL_DESTROY`.
pub fn anvil_destroy() -> Holder<SoundEvent> {
    Holder::Key(key("block.anvil.destroy"))
}

/// `SoundEvents.ANVIL_FALL`.
pub fn anvil_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.anvil.fall"))
}

/// `SoundEvents.ANVIL_HIT`.
pub fn anvil_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.anvil.hit"))
}

/// `SoundEvents.ANVIL_LAND`.
pub fn anvil_land() -> Holder<SoundEvent> {
    Holder::Key(key("block.anvil.land"))
}

/// `SoundEvents.ANVIL_PLACE`.
pub fn anvil_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.anvil.place"))
}

/// `SoundEvents.ANVIL_STEP`.
pub fn anvil_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.anvil.step"))
}

/// `SoundEvents.ANVIL_USE`.
pub fn anvil_use() -> Holder<SoundEvent> {
    Holder::Key(key("block.anvil.use"))
}

/// `SoundEvents.ARMADILLO_EAT`.
pub fn armadillo_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.eat"))
}

/// `SoundEvents.ARMADILLO_HURT`.
pub fn armadillo_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.hurt"))
}

/// `SoundEvents.ARMADILLO_HURT_REDUCED`.
pub fn armadillo_hurt_reduced() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.hurt_reduced"))
}

/// `SoundEvents.ARMADILLO_AMBIENT`.
pub fn armadillo_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.ambient"))
}

/// `SoundEvents.ARMADILLO_STEP`.
pub fn armadillo_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.step"))
}

/// `SoundEvents.ARMADILLO_DEATH`.
pub fn armadillo_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.death"))
}

/// `SoundEvents.ARMADILLO_ROLL`.
pub fn armadillo_roll() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.roll"))
}

/// `SoundEvents.ARMADILLO_LAND`.
pub fn armadillo_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.land"))
}

/// `SoundEvents.ARMADILLO_SCUTE_DROP`.
pub fn armadillo_scute_drop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.scute_drop"))
}

/// `SoundEvents.ARMADILLO_UNROLL_FINISH`.
pub fn armadillo_unroll_finish() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.unroll_finish"))
}

/// `SoundEvents.ARMADILLO_PEEK`.
pub fn armadillo_peek() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.peek"))
}

/// `SoundEvents.ARMADILLO_UNROLL_START`.
pub fn armadillo_unroll_start() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.unroll_start"))
}

/// `SoundEvents.ARMADILLO_BRUSH`.
pub fn armadillo_brush() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armadillo.brush"))
}

/// `SoundEvents.ARMOR_UNEQUIP_WOLF`.
pub fn armor_unequip_wolf() -> Holder<SoundEvent> {
    Holder::Key(key("item.armor.unequip_wolf"))
}

/// `SoundEvents.ARMOR_EQUIP_NAUTILUS`.
pub fn armor_equip_nautilus() -> Holder<SoundEvent> {
    Holder::Key(key("item.armor.equip_nautilus"))
}

/// `SoundEvents.ARMOR_UNEQUIP_NAUTILUS`.
pub fn armor_unequip_nautilus() -> Holder<SoundEvent> {
    Holder::Key(key("item.armor.unequip_nautilus"))
}

/// `SoundEvents.ARMOR_STAND_BREAK`.
pub fn armor_stand_break() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armor_stand.break"))
}

/// `SoundEvents.ARMOR_STAND_FALL`.
pub fn armor_stand_fall() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armor_stand.fall"))
}

/// `SoundEvents.ARMOR_STAND_HIT`.
pub fn armor_stand_hit() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armor_stand.hit"))
}

/// `SoundEvents.ARMOR_STAND_PLACE`.
pub fn armor_stand_place() -> Holder<SoundEvent> {
    Holder::Key(key("entity.armor_stand.place"))
}

/// `SoundEvents.ARROW_HIT`.
pub fn arrow_hit() -> Holder<SoundEvent> {
    Holder::Key(key("entity.arrow.hit"))
}

/// `SoundEvents.ARROW_HIT_PLAYER`.
pub fn arrow_hit_player() -> Holder<SoundEvent> {
    Holder::Key(key("entity.arrow.hit_player"))
}

/// `SoundEvents.ARROW_SHOOT`.
pub fn arrow_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.arrow.shoot"))
}

/// `SoundEvents.AXE_STRIP`.
pub fn axe_strip() -> Holder<SoundEvent> {
    Holder::Key(key("item.axe.strip"))
}

/// `SoundEvents.AXE_SCRAPE`.
pub fn axe_scrape() -> Holder<SoundEvent> {
    Holder::Key(key("item.axe.scrape"))
}

/// `SoundEvents.AXE_WAX_OFF`.
pub fn axe_wax_off() -> Holder<SoundEvent> {
    Holder::Key(key("item.axe.wax_off"))
}

/// `SoundEvents.AXOLOTL_ATTACK`.
pub fn axolotl_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.axolotl.attack"))
}

/// `SoundEvents.AXOLOTL_DEATH`.
pub fn axolotl_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.axolotl.death"))
}

/// `SoundEvents.AXOLOTL_HURT`.
pub fn axolotl_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.axolotl.hurt"))
}

/// `SoundEvents.AXOLOTL_IDLE_AIR`.
pub fn axolotl_idle_air() -> Holder<SoundEvent> {
    Holder::Key(key("entity.axolotl.idle_air"))
}

/// `SoundEvents.AXOLOTL_IDLE_WATER`.
pub fn axolotl_idle_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.axolotl.idle_water"))
}

/// `SoundEvents.AXOLOTL_SPLASH`.
pub fn axolotl_splash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.axolotl.splash"))
}

/// `SoundEvents.AXOLOTL_SWIM`.
pub fn axolotl_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.axolotl.swim"))
}

/// `SoundEvents.AZALEA_BREAK`.
pub fn azalea_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea.break"))
}

/// `SoundEvents.AZALEA_FALL`.
pub fn azalea_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea.fall"))
}

/// `SoundEvents.AZALEA_HIT`.
pub fn azalea_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea.hit"))
}

/// `SoundEvents.AZALEA_PLACE`.
pub fn azalea_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea.place"))
}

/// `SoundEvents.AZALEA_STEP`.
pub fn azalea_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea.step"))
}

/// `SoundEvents.AZALEA_LEAVES_BREAK`.
pub fn azalea_leaves_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea_leaves.break"))
}

/// `SoundEvents.AZALEA_LEAVES_FALL`.
pub fn azalea_leaves_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea_leaves.fall"))
}

/// `SoundEvents.AZALEA_LEAVES_HIT`.
pub fn azalea_leaves_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea_leaves.hit"))
}

/// `SoundEvents.AZALEA_LEAVES_PLACE`.
pub fn azalea_leaves_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea_leaves.place"))
}

/// `SoundEvents.AZALEA_LEAVES_STEP`.
pub fn azalea_leaves_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.azalea_leaves.step"))
}

/// `SoundEvents.BABY_NAUTILUS_AMBIENT`.
pub fn baby_nautilus_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.baby_nautilus.ambient"))
}

/// `SoundEvents.BABY_NAUTILUS_AMBIENT_ON_LAND`.
pub fn baby_nautilus_ambient_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.baby_nautilus.ambient_land"))
}

/// `SoundEvents.BABY_NAUTILUS_DEATH`.
pub fn baby_nautilus_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.baby_nautilus.death"))
}

/// `SoundEvents.BABY_NAUTILUS_DEATH_ON_LAND`.
pub fn baby_nautilus_death_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.baby_nautilus.death_land"))
}

/// `SoundEvents.BABY_NAUTILUS_EAT`.
pub fn baby_nautilus_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.baby_nautilus.eat"))
}

/// `SoundEvents.BABY_NAUTILUS_HURT`.
pub fn baby_nautilus_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.baby_nautilus.hurt"))
}

/// `SoundEvents.BABY_NAUTILUS_HURT_ON_LAND`.
pub fn baby_nautilus_hurt_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.baby_nautilus.hurt_land"))
}

/// `SoundEvents.NAUTILUS_RIDING`.
pub fn nautilus_riding() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.riding"))
}

/// `SoundEvents.BABY_NAUTILUS_SWIM`.
pub fn baby_nautilus_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.baby_nautilus.swim"))
}

/// `SoundEvents.BAMBOO_BREAK`.
pub fn bamboo_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo.break"))
}

/// `SoundEvents.BAMBOO_FALL`.
pub fn bamboo_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo.fall"))
}

/// `SoundEvents.BAMBOO_HIT`.
pub fn bamboo_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo.hit"))
}

/// `SoundEvents.BAMBOO_PLACE`.
pub fn bamboo_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo.place"))
}

/// `SoundEvents.BAMBOO_STEP`.
pub fn bamboo_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo.step"))
}

/// `SoundEvents.BAMBOO_SAPLING_BREAK`.
pub fn bamboo_sapling_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_sapling.break"))
}

/// `SoundEvents.BAMBOO_SAPLING_HIT`.
pub fn bamboo_sapling_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_sapling.hit"))
}

/// `SoundEvents.BAMBOO_SAPLING_PLACE`.
pub fn bamboo_sapling_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_sapling.place"))
}

/// `SoundEvents.BAMBOO_WOOD_BREAK`.
pub fn bamboo_wood_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood.break"))
}

/// `SoundEvents.BAMBOO_WOOD_FALL`.
pub fn bamboo_wood_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood.fall"))
}

/// `SoundEvents.BAMBOO_WOOD_HIT`.
pub fn bamboo_wood_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood.hit"))
}

/// `SoundEvents.BAMBOO_WOOD_PLACE`.
pub fn bamboo_wood_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood.place"))
}

/// `SoundEvents.BAMBOO_WOOD_STEP`.
pub fn bamboo_wood_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood.step"))
}

/// `SoundEvents.BAMBOO_WOOD_DOOR_CLOSE`.
pub fn bamboo_wood_door_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_door.close"))
}

/// `SoundEvents.BAMBOO_WOOD_DOOR_OPEN`.
pub fn bamboo_wood_door_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_door.open"))
}

/// `SoundEvents.BAMBOO_WOOD_TRAPDOOR_CLOSE`.
pub fn bamboo_wood_trapdoor_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_trapdoor.close"))
}

/// `SoundEvents.BAMBOO_WOOD_TRAPDOOR_OPEN`.
pub fn bamboo_wood_trapdoor_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_trapdoor.open"))
}

/// `SoundEvents.BAMBOO_WOOD_BUTTON_CLICK_OFF`.
pub fn bamboo_wood_button_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_button.click_off"))
}

/// `SoundEvents.BAMBOO_WOOD_BUTTON_CLICK_ON`.
pub fn bamboo_wood_button_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_button.click_on"))
}

/// `SoundEvents.BAMBOO_WOOD_PRESSURE_PLATE_CLICK_OFF`.
pub fn bamboo_wood_pressure_plate_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_pressure_plate.click_off"))
}

/// `SoundEvents.BAMBOO_WOOD_PRESSURE_PLATE_CLICK_ON`.
pub fn bamboo_wood_pressure_plate_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_pressure_plate.click_on"))
}

/// `SoundEvents.BAMBOO_WOOD_FENCE_GATE_CLOSE`.
pub fn bamboo_wood_fence_gate_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_fence_gate.close"))
}

/// `SoundEvents.BAMBOO_WOOD_FENCE_GATE_OPEN`.
pub fn bamboo_wood_fence_gate_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_fence_gate.open"))
}

/// `SoundEvents.BARREL_CLOSE`.
pub fn barrel_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.barrel.close"))
}

/// `SoundEvents.BARREL_OPEN`.
pub fn barrel_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.barrel.open"))
}

/// `SoundEvents.BASALT_BREAK`.
pub fn basalt_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.basalt.break"))
}

/// `SoundEvents.BASALT_STEP`.
pub fn basalt_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.basalt.step"))
}

/// `SoundEvents.BASALT_PLACE`.
pub fn basalt_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.basalt.place"))
}

/// `SoundEvents.BASALT_HIT`.
pub fn basalt_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.basalt.hit"))
}

/// `SoundEvents.BASALT_FALL`.
pub fn basalt_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.basalt.fall"))
}

/// `SoundEvents.BAT_AMBIENT`.
pub fn bat_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bat.ambient"))
}

/// `SoundEvents.BAT_DEATH`.
pub fn bat_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bat.death"))
}

/// `SoundEvents.BAT_HURT`.
pub fn bat_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bat.hurt"))
}

/// `SoundEvents.BAT_LOOP`.
pub fn bat_loop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bat.loop"))
}

/// `SoundEvents.BAT_TAKEOFF`.
pub fn bat_takeoff() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bat.takeoff"))
}

/// `SoundEvents.BEACON_ACTIVATE`.
pub fn beacon_activate() -> Holder<SoundEvent> {
    Holder::Key(key("block.beacon.activate"))
}

/// `SoundEvents.BEACON_AMBIENT`.
pub fn beacon_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.beacon.ambient"))
}

/// `SoundEvents.BEACON_DEACTIVATE`.
pub fn beacon_deactivate() -> Holder<SoundEvent> {
    Holder::Key(key("block.beacon.deactivate"))
}

/// `SoundEvents.BEACON_POWER_SELECT`.
pub fn beacon_power_select() -> Holder<SoundEvent> {
    Holder::Key(key("block.beacon.power_select"))
}

/// `SoundEvents.BEE_DEATH`.
pub fn bee_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bee.death"))
}

/// `SoundEvents.BEE_HURT`.
pub fn bee_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bee.hurt"))
}

/// `SoundEvents.BEE_LOOP_AGGRESSIVE`.
pub fn bee_loop_aggressive() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bee.loop_aggressive"))
}

/// `SoundEvents.BEE_LOOP`.
pub fn bee_loop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bee.loop"))
}

/// `SoundEvents.BEE_STING`.
pub fn bee_sting() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bee.sting"))
}

/// `SoundEvents.BEE_POLLINATE`.
pub fn bee_pollinate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bee.pollinate"))
}

/// `SoundEvents.BEEHIVE_DRIP`.
pub fn beehive_drip() -> Holder<SoundEvent> {
    Holder::Key(key("block.beehive.drip"))
}

/// `SoundEvents.BEEHIVE_ENTER`.
pub fn beehive_enter() -> Holder<SoundEvent> {
    Holder::Key(key("block.beehive.enter"))
}

/// `SoundEvents.BEEHIVE_EXIT`.
pub fn beehive_exit() -> Holder<SoundEvent> {
    Holder::Key(key("block.beehive.exit"))
}

/// `SoundEvents.BEEHIVE_SHEAR`.
pub fn beehive_shear() -> Holder<SoundEvent> {
    Holder::Key(key("block.beehive.shear"))
}

/// `SoundEvents.BEEHIVE_WORK`.
pub fn beehive_work() -> Holder<SoundEvent> {
    Holder::Key(key("block.beehive.work"))
}

/// `SoundEvents.BELL_BLOCK`.
pub fn bell_block() -> Holder<SoundEvent> {
    Holder::Key(key("block.bell.use"))
}

/// `SoundEvents.BELL_RESONATE`.
pub fn bell_resonate() -> Holder<SoundEvent> {
    Holder::Key(key("block.bell.resonate"))
}

/// `SoundEvents.BIG_DRIPLEAF_BREAK`.
pub fn big_dripleaf_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.big_dripleaf.break"))
}

/// `SoundEvents.BIG_DRIPLEAF_FALL`.
pub fn big_dripleaf_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.big_dripleaf.fall"))
}

/// `SoundEvents.BIG_DRIPLEAF_HIT`.
pub fn big_dripleaf_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.big_dripleaf.hit"))
}

/// `SoundEvents.BIG_DRIPLEAF_PLACE`.
pub fn big_dripleaf_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.big_dripleaf.place"))
}

/// `SoundEvents.BIG_DRIPLEAF_STEP`.
pub fn big_dripleaf_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.big_dripleaf.step"))
}

/// `SoundEvents.BLAZE_AMBIENT`.
pub fn blaze_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.blaze.ambient"))
}

/// `SoundEvents.BLAZE_BURN`.
pub fn blaze_burn() -> Holder<SoundEvent> {
    Holder::Key(key("entity.blaze.burn"))
}

/// `SoundEvents.BLAZE_DEATH`.
pub fn blaze_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.blaze.death"))
}

/// `SoundEvents.BLAZE_HURT`.
pub fn blaze_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.blaze.hurt"))
}

/// `SoundEvents.BLAZE_SHOOT`.
pub fn blaze_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.blaze.shoot"))
}

/// `SoundEvents.BOAT_PADDLE_LAND`.
pub fn boat_paddle_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.boat.paddle_land"))
}

/// `SoundEvents.BOAT_PADDLE_WATER`.
pub fn boat_paddle_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.boat.paddle_water"))
}

/// `SoundEvents.BOGGED_AMBIENT`.
pub fn bogged_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bogged.ambient"))
}

/// `SoundEvents.BOGGED_DEATH`.
pub fn bogged_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bogged.death"))
}

/// `SoundEvents.BOGGED_HURT`.
pub fn bogged_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bogged.hurt"))
}

/// `SoundEvents.BOGGED_SHEAR`.
pub fn bogged_shear() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bogged.shear"))
}

/// `SoundEvents.BOGGED_STEP`.
pub fn bogged_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.bogged.step"))
}

/// `SoundEvents.BONE_BLOCK_BREAK`.
pub fn bone_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.bone_block.break"))
}

/// `SoundEvents.BONE_BLOCK_FALL`.
pub fn bone_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.bone_block.fall"))
}

/// `SoundEvents.BONE_BLOCK_HIT`.
pub fn bone_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.bone_block.hit"))
}

/// `SoundEvents.BONE_BLOCK_PLACE`.
pub fn bone_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.bone_block.place"))
}

/// `SoundEvents.BONE_BLOCK_STEP`.
pub fn bone_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.bone_block.step"))
}

/// `SoundEvents.BONE_MEAL_USE`.
pub fn bone_meal_use() -> Holder<SoundEvent> {
    Holder::Key(key("item.bone_meal.use"))
}

/// `SoundEvents.BOOK_PAGE_TURN`.
pub fn book_page_turn() -> Holder<SoundEvent> {
    Holder::Key(key("item.book.page_turn"))
}

/// `SoundEvents.BOOK_PUT`.
pub fn book_put() -> Holder<SoundEvent> {
    Holder::Key(key("item.book.put"))
}

/// `SoundEvents.BLASTFURNACE_FIRE_CRACKLE`.
pub fn blastfurnace_fire_crackle() -> Holder<SoundEvent> {
    Holder::Key(key("block.blastfurnace.fire_crackle"))
}

/// `SoundEvents.BOTTLE_EMPTY`.
pub fn bottle_empty() -> Holder<SoundEvent> {
    Holder::Key(key("item.bottle.empty"))
}

/// `SoundEvents.BOTTLE_FILL`.
pub fn bottle_fill() -> Holder<SoundEvent> {
    Holder::Key(key("item.bottle.fill"))
}

/// `SoundEvents.BOTTLE_FILL_DRAGONBREATH`.
pub fn bottle_fill_dragonbreath() -> Holder<SoundEvent> {
    Holder::Key(key("item.bottle.fill_dragonbreath"))
}

/// `SoundEvents.BREEZE_CHARGE`.
pub fn breeze_charge() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.charge"))
}

/// `SoundEvents.BREEZE_DEFLECT`.
pub fn breeze_deflect() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.deflect"))
}

/// `SoundEvents.BREEZE_INHALE`.
pub fn breeze_inhale() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.inhale"))
}

/// `SoundEvents.BREEZE_IDLE_GROUND`.
pub fn breeze_idle_ground() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.idle_ground"))
}

/// `SoundEvents.BREEZE_IDLE_AIR`.
pub fn breeze_idle_air() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.idle_air"))
}

/// `SoundEvents.BREEZE_SHOOT`.
pub fn breeze_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.shoot"))
}

/// `SoundEvents.BREEZE_JUMP`.
pub fn breeze_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.jump"))
}

/// `SoundEvents.BREEZE_LAND`.
pub fn breeze_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.land"))
}

/// `SoundEvents.BREEZE_SLIDE`.
pub fn breeze_slide() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.slide"))
}

/// `SoundEvents.BREEZE_DEATH`.
pub fn breeze_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.death"))
}

/// `SoundEvents.BREEZE_HURT`.
pub fn breeze_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.hurt"))
}

/// `SoundEvents.BREEZE_WHIRL`.
pub fn breeze_whirl() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.whirl"))
}

/// `SoundEvents.BREEZE_WIND_CHARGE_BURST`.
pub fn breeze_wind_charge_burst() -> Holder<SoundEvent> {
    Holder::Key(key("entity.breeze.wind_burst"))
}

/// `SoundEvents.BREWING_STAND_BREW`.
pub fn brewing_stand_brew() -> Holder<SoundEvent> {
    Holder::Key(key("block.brewing_stand.brew"))
}

/// `SoundEvents.BRUSH_GENERIC`.
pub fn brush_generic() -> Holder<SoundEvent> {
    Holder::Key(key("item.brush.brushing.generic"))
}

/// `SoundEvents.BRUSH_SAND`.
pub fn brush_sand() -> Holder<SoundEvent> {
    Holder::Key(key("item.brush.brushing.sand"))
}

/// `SoundEvents.BRUSH_GRAVEL`.
pub fn brush_gravel() -> Holder<SoundEvent> {
    Holder::Key(key("item.brush.brushing.gravel"))
}

/// `SoundEvents.BRUSH_SAND_COMPLETED`.
pub fn brush_sand_completed() -> Holder<SoundEvent> {
    Holder::Key(key("item.brush.brushing.sand.complete"))
}

/// `SoundEvents.BRUSH_GRAVEL_COMPLETED`.
pub fn brush_gravel_completed() -> Holder<SoundEvent> {
    Holder::Key(key("item.brush.brushing.gravel.complete"))
}

/// `SoundEvents.BUBBLE_COLUMN_BUBBLE_POP`.
pub fn bubble_column_bubble_pop() -> Holder<SoundEvent> {
    Holder::Key(key("block.bubble_column.bubble_pop"))
}

/// `SoundEvents.BUBBLE_COLUMN_UPWARDS_AMBIENT`.
pub fn bubble_column_upwards_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.bubble_column.upwards_ambient"))
}

/// `SoundEvents.BUBBLE_COLUMN_UPWARDS_INSIDE`.
pub fn bubble_column_upwards_inside() -> Holder<SoundEvent> {
    Holder::Key(key("block.bubble_column.upwards_inside"))
}

/// `SoundEvents.BUBBLE_COLUMN_WHIRLPOOL_AMBIENT`.
pub fn bubble_column_whirlpool_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.bubble_column.whirlpool_ambient"))
}

/// `SoundEvents.BUBBLE_COLUMN_WHIRLPOOL_INSIDE`.
pub fn bubble_column_whirlpool_inside() -> Holder<SoundEvent> {
    Holder::Key(key("block.bubble_column.whirlpool_inside"))
}

/// `SoundEvents.BUBBLE_POP`.
pub fn bubble_pop() -> Holder<SoundEvent> {
    Holder::Key(key("ui.hud.bubble_pop"))
}

/// `SoundEvents.BUCKET_EMPTY`.
pub fn bucket_empty() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.empty"))
}

/// `SoundEvents.BUCKET_EMPTY_AXOLOTL`.
pub fn bucket_empty_axolotl() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.empty_axolotl"))
}

/// `SoundEvents.BUCKET_EMPTY_FISH`.
pub fn bucket_empty_fish() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.empty_fish"))
}

/// `SoundEvents.BUCKET_EMPTY_LAVA`.
pub fn bucket_empty_lava() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.empty_lava"))
}

/// `SoundEvents.BUCKET_EMPTY_POWDER_SNOW`.
pub fn bucket_empty_powder_snow() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.empty_powder_snow"))
}

/// `SoundEvents.BUCKET_EMPTY_TADPOLE`.
pub fn bucket_empty_tadpole() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.empty_tadpole"))
}

/// `SoundEvents.BUCKET_FILL`.
pub fn bucket_fill() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.fill"))
}

/// `SoundEvents.BUCKET_FILL_AXOLOTL`.
pub fn bucket_fill_axolotl() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.fill_axolotl"))
}

/// `SoundEvents.BUCKET_FILL_FISH`.
pub fn bucket_fill_fish() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.fill_fish"))
}

/// `SoundEvents.BUCKET_FILL_LAVA`.
pub fn bucket_fill_lava() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.fill_lava"))
}

/// `SoundEvents.BUCKET_FILL_POWDER_SNOW`.
pub fn bucket_fill_powder_snow() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.fill_powder_snow"))
}

/// `SoundEvents.BUCKET_FILL_TADPOLE`.
pub fn bucket_fill_tadpole() -> Holder<SoundEvent> {
    Holder::Key(key("item.bucket.fill_tadpole"))
}

/// `SoundEvents.BUNDLE_DROP_CONTENTS`.
pub fn bundle_drop_contents() -> Holder<SoundEvent> {
    Holder::Key(key("item.bundle.drop_contents"))
}

/// `SoundEvents.BUNDLE_INSERT`.
pub fn bundle_insert() -> Holder<SoundEvent> {
    Holder::Key(key("item.bundle.insert"))
}

/// `SoundEvents.BUNDLE_INSERT_FAIL`.
pub fn bundle_insert_fail() -> Holder<SoundEvent> {
    Holder::Key(key("item.bundle.insert_fail"))
}

/// `SoundEvents.BUNDLE_REMOVE_ONE`.
pub fn bundle_remove_one() -> Holder<SoundEvent> {
    Holder::Key(key("item.bundle.remove_one"))
}

/// `SoundEvents.CACTUS_FLOWER_BREAK`.
pub fn cactus_flower_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.cactus_flower.break"))
}

/// `SoundEvents.CACTUS_FLOWER_PLACE`.
pub fn cactus_flower_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.cactus_flower.place"))
}

/// `SoundEvents.CAKE_ADD_CANDLE`.
pub fn cake_add_candle() -> Holder<SoundEvent> {
    Holder::Key(key("block.cake.add_candle"))
}

/// `SoundEvents.CALCITE_BREAK`.
pub fn calcite_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.calcite.break"))
}

/// `SoundEvents.CALCITE_STEP`.
pub fn calcite_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.calcite.step"))
}

/// `SoundEvents.CALCITE_PLACE`.
pub fn calcite_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.calcite.place"))
}

/// `SoundEvents.CALCITE_HIT`.
pub fn calcite_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.calcite.hit"))
}

/// `SoundEvents.CALCITE_FALL`.
pub fn calcite_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.calcite.fall"))
}

/// `SoundEvents.CAMEL_HUSK_AMBIENT`.
pub fn camel_husk_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.ambient"))
}

/// `SoundEvents.CAMEL_HUSK_DASH`.
pub fn camel_husk_dash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.dash"))
}

/// `SoundEvents.CAMEL_HUSK_DASH_READY`.
pub fn camel_husk_dash_ready() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.dash_ready"))
}

/// `SoundEvents.CAMEL_HUSK_DEATH`.
pub fn camel_husk_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.death"))
}

/// `SoundEvents.CAMEL_HUSK_EAT`.
pub fn camel_husk_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.eat"))
}

/// `SoundEvents.CAMEL_HUSK_HURT`.
pub fn camel_husk_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.hurt"))
}

/// `SoundEvents.CAMEL_HUSK_SADDLE`.
pub fn camel_husk_saddle() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.saddle"))
}

/// `SoundEvents.CAMEL_HUSK_SIT`.
pub fn camel_husk_sit() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.sit"))
}

/// `SoundEvents.CAMEL_HUSK_STAND`.
pub fn camel_husk_stand() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.stand"))
}

/// `SoundEvents.CAMEL_HUSK_STEP`.
pub fn camel_husk_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.step"))
}

/// `SoundEvents.CAMEL_HUSK_STEP_SAND`.
pub fn camel_husk_step_sand() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel_husk.step_sand"))
}

/// `SoundEvents.CAMEL_AMBIENT`.
pub fn camel_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.ambient"))
}

/// `SoundEvents.CAMEL_DASH`.
pub fn camel_dash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.dash"))
}

/// `SoundEvents.CAMEL_DASH_READY`.
pub fn camel_dash_ready() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.dash_ready"))
}

/// `SoundEvents.CAMEL_DEATH`.
pub fn camel_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.death"))
}

/// `SoundEvents.CAMEL_EAT`.
pub fn camel_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.eat"))
}

/// `SoundEvents.CAMEL_HURT`.
pub fn camel_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.hurt"))
}

/// `SoundEvents.CAMEL_SADDLE`.
pub fn camel_saddle() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.saddle"))
}

/// `SoundEvents.CAMEL_SIT`.
pub fn camel_sit() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.sit"))
}

/// `SoundEvents.CAMEL_STAND`.
pub fn camel_stand() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.stand"))
}

/// `SoundEvents.CAMEL_STEP`.
pub fn camel_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.step"))
}

/// `SoundEvents.CAMEL_STEP_SAND`.
pub fn camel_step_sand() -> Holder<SoundEvent> {
    Holder::Key(key("entity.camel.step_sand"))
}

/// `SoundEvents.CAMPFIRE_CRACKLE`.
pub fn campfire_crackle() -> Holder<SoundEvent> {
    Holder::Key(key("block.campfire.crackle"))
}

/// `SoundEvents.CANDLE_AMBIENT`.
pub fn candle_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.candle.ambient"))
}

/// `SoundEvents.CANDLE_BREAK`.
pub fn candle_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.candle.break"))
}

/// `SoundEvents.CANDLE_EXTINGUISH`.
pub fn candle_extinguish() -> Holder<SoundEvent> {
    Holder::Key(key("block.candle.extinguish"))
}

/// `SoundEvents.CANDLE_FALL`.
pub fn candle_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.candle.fall"))
}

/// `SoundEvents.CANDLE_HIT`.
pub fn candle_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.candle.hit"))
}

/// `SoundEvents.CANDLE_PLACE`.
pub fn candle_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.candle.place"))
}

/// `SoundEvents.CANDLE_STEP`.
pub fn candle_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.candle.step"))
}

/// `SoundEvents.CAT_AMBIENT`.
pub fn cat_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.ambient"))
}

/// `SoundEvents.CAT_STRAY_AMBIENT`.
pub fn cat_stray_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.stray_ambient"))
}

/// `SoundEvents.CAT_DEATH`.
pub fn cat_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.death"))
}

/// `SoundEvents.CAT_EAT`.
pub fn cat_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.eat"))
}

/// `SoundEvents.CAT_HISS`.
pub fn cat_hiss() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.hiss"))
}

/// `SoundEvents.CAT_BEG_FOR_FOOD`.
pub fn cat_beg_for_food() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.beg_for_food"))
}

/// `SoundEvents.CAT_HURT`.
pub fn cat_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.hurt"))
}

/// `SoundEvents.CAT_PURR`.
pub fn cat_purr() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.purr"))
}

/// `SoundEvents.CAT_PURREOW`.
pub fn cat_purreow() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cat.purreow"))
}

/// `SoundEvents.CAVE_VINES_BREAK`.
pub fn cave_vines_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.cave_vines.break"))
}

/// `SoundEvents.CAVE_VINES_FALL`.
pub fn cave_vines_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.cave_vines.fall"))
}

/// `SoundEvents.CAVE_VINES_HIT`.
pub fn cave_vines_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.cave_vines.hit"))
}

/// `SoundEvents.CAVE_VINES_PLACE`.
pub fn cave_vines_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.cave_vines.place"))
}

/// `SoundEvents.CAVE_VINES_STEP`.
pub fn cave_vines_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.cave_vines.step"))
}

/// `SoundEvents.CAVE_VINES_PICK_BERRIES`.
pub fn cave_vines_pick_berries() -> Holder<SoundEvent> {
    Holder::Key(key("block.cave_vines.pick_berries"))
}

/// `SoundEvents.CHAIN_BREAK`.
pub fn chain_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.chain.break"))
}

/// `SoundEvents.CHAIN_FALL`.
pub fn chain_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.chain.fall"))
}

/// `SoundEvents.CHAIN_HIT`.
pub fn chain_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.chain.hit"))
}

/// `SoundEvents.CHAIN_PLACE`.
pub fn chain_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.chain.place"))
}

/// `SoundEvents.CHAIN_STEP`.
pub fn chain_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.chain.step"))
}

/// `SoundEvents.CHERRY_WOOD_BREAK`.
pub fn cherry_wood_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood.break"))
}

/// `SoundEvents.CHERRY_WOOD_FALL`.
pub fn cherry_wood_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood.fall"))
}

/// `SoundEvents.CHERRY_WOOD_HIT`.
pub fn cherry_wood_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood.hit"))
}

/// `SoundEvents.CHERRY_WOOD_PLACE`.
pub fn cherry_wood_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood.place"))
}

/// `SoundEvents.CHERRY_WOOD_STEP`.
pub fn cherry_wood_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood.step"))
}

/// `SoundEvents.CHERRY_SAPLING_BREAK`.
pub fn cherry_sapling_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_sapling.break"))
}

/// `SoundEvents.CHERRY_SAPLING_FALL`.
pub fn cherry_sapling_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_sapling.fall"))
}

/// `SoundEvents.CHERRY_SAPLING_HIT`.
pub fn cherry_sapling_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_sapling.hit"))
}

/// `SoundEvents.CHERRY_SAPLING_PLACE`.
pub fn cherry_sapling_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_sapling.place"))
}

/// `SoundEvents.CHERRY_SAPLING_STEP`.
pub fn cherry_sapling_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_sapling.step"))
}

/// `SoundEvents.CHERRY_LEAVES_BREAK`.
pub fn cherry_leaves_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_leaves.break"))
}

/// `SoundEvents.CHERRY_LEAVES_FALL`.
pub fn cherry_leaves_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_leaves.fall"))
}

/// `SoundEvents.CHERRY_LEAVES_HIT`.
pub fn cherry_leaves_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_leaves.hit"))
}

/// `SoundEvents.CHERRY_LEAVES_PLACE`.
pub fn cherry_leaves_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_leaves.place"))
}

/// `SoundEvents.CHERRY_LEAVES_STEP`.
pub fn cherry_leaves_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_leaves.step"))
}

/// `SoundEvents.CHERRY_WOOD_HANGING_SIGN_STEP`.
pub fn cherry_wood_hanging_sign_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_hanging_sign.step"))
}

/// `SoundEvents.CHERRY_WOOD_HANGING_SIGN_BREAK`.
pub fn cherry_wood_hanging_sign_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_hanging_sign.break"))
}

/// `SoundEvents.CHERRY_WOOD_HANGING_SIGN_FALL`.
pub fn cherry_wood_hanging_sign_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_hanging_sign.fall"))
}

/// `SoundEvents.CHERRY_WOOD_HANGING_SIGN_HIT`.
pub fn cherry_wood_hanging_sign_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_hanging_sign.hit"))
}

/// `SoundEvents.CHERRY_WOOD_HANGING_SIGN_PLACE`.
pub fn cherry_wood_hanging_sign_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_hanging_sign.place"))
}

/// `SoundEvents.CHERRY_WOOD_DOOR_CLOSE`.
pub fn cherry_wood_door_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_door.close"))
}

/// `SoundEvents.CHERRY_WOOD_DOOR_OPEN`.
pub fn cherry_wood_door_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_door.open"))
}

/// `SoundEvents.CHERRY_WOOD_TRAPDOOR_CLOSE`.
pub fn cherry_wood_trapdoor_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_trapdoor.close"))
}

/// `SoundEvents.CHERRY_WOOD_TRAPDOOR_OPEN`.
pub fn cherry_wood_trapdoor_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_trapdoor.open"))
}

/// `SoundEvents.CHERRY_WOOD_BUTTON_CLICK_OFF`.
pub fn cherry_wood_button_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_button.click_off"))
}

/// `SoundEvents.CHERRY_WOOD_BUTTON_CLICK_ON`.
pub fn cherry_wood_button_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_button.click_on"))
}

/// `SoundEvents.CHERRY_WOOD_PRESSURE_PLATE_CLICK_OFF`.
pub fn cherry_wood_pressure_plate_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_pressure_plate.click_off"))
}

/// `SoundEvents.CHERRY_WOOD_PRESSURE_PLATE_CLICK_ON`.
pub fn cherry_wood_pressure_plate_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_pressure_plate.click_on"))
}

/// `SoundEvents.CHERRY_WOOD_FENCE_GATE_CLOSE`.
pub fn cherry_wood_fence_gate_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_fence_gate.close"))
}

/// `SoundEvents.CHERRY_WOOD_FENCE_GATE_OPEN`.
pub fn cherry_wood_fence_gate_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.cherry_wood_fence_gate.open"))
}

/// `SoundEvents.CHEST_CLOSE`.
pub fn chest_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.chest.close"))
}

/// `SoundEvents.CHEST_LOCKED`.
pub fn chest_locked() -> Holder<SoundEvent> {
    Holder::Key(key("block.chest.locked"))
}

/// `SoundEvents.CHEST_OPEN`.
pub fn chest_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.chest.open"))
}

/// `SoundEvents.CHICKEN_AMBIENT`.
pub fn chicken_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.chicken.ambient"))
}

/// `SoundEvents.CHICKEN_DEATH`.
pub fn chicken_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.chicken.death"))
}

/// `SoundEvents.CHICKEN_EGG`.
pub fn chicken_egg() -> Holder<SoundEvent> {
    Holder::Key(key("entity.chicken.egg"))
}

/// `SoundEvents.CHICKEN_HURT`.
pub fn chicken_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.chicken.hurt"))
}

/// `SoundEvents.CHICKEN_STEP`.
pub fn chicken_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.chicken.step"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_BREAK`.
pub fn chiseled_bookshelf_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.break"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_FALL`.
pub fn chiseled_bookshelf_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.fall"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_HIT`.
pub fn chiseled_bookshelf_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.hit"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_INSERT`.
pub fn chiseled_bookshelf_insert() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.insert"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_INSERT_ENCHANTED`.
pub fn chiseled_bookshelf_insert_enchanted() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.insert.enchanted"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_STEP`.
pub fn chiseled_bookshelf_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.step"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_PICKUP`.
pub fn chiseled_bookshelf_pickup() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.pickup"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_PICKUP_ENCHANTED`.
pub fn chiseled_bookshelf_pickup_enchanted() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.pickup.enchanted"))
}

/// `SoundEvents.CHISELED_BOOKSHELF_PLACE`.
pub fn chiseled_bookshelf_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.chiseled_bookshelf.place"))
}

/// `SoundEvents.CHORUS_FLOWER_DEATH`.
pub fn chorus_flower_death() -> Holder<SoundEvent> {
    Holder::Key(key("block.chorus_flower.death"))
}

/// `SoundEvents.CHORUS_FLOWER_GROW`.
pub fn chorus_flower_grow() -> Holder<SoundEvent> {
    Holder::Key(key("block.chorus_flower.grow"))
}

/// `SoundEvents.CHORUS_FRUIT_TELEPORT`.
pub fn chorus_fruit_teleport() -> Holder<SoundEvent> {
    Holder::Key(key("item.chorus_fruit.teleport"))
}

/// `SoundEvents.COBWEB_BREAK`.
pub fn cobweb_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.cobweb.break"))
}

/// `SoundEvents.COBWEB_STEP`.
pub fn cobweb_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.cobweb.step"))
}

/// `SoundEvents.COBWEB_PLACE`.
pub fn cobweb_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.cobweb.place"))
}

/// `SoundEvents.COBWEB_HIT`.
pub fn cobweb_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.cobweb.hit"))
}

/// `SoundEvents.COBWEB_FALL`.
pub fn cobweb_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.cobweb.fall"))
}

/// `SoundEvents.COD_AMBIENT`.
pub fn cod_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cod.ambient"))
}

/// `SoundEvents.COD_DEATH`.
pub fn cod_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cod.death"))
}

/// `SoundEvents.COD_FLOP`.
pub fn cod_flop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cod.flop"))
}

/// `SoundEvents.COD_HURT`.
pub fn cod_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cod.hurt"))
}

/// `SoundEvents.COMPARATOR_CLICK`.
pub fn comparator_click() -> Holder<SoundEvent> {
    Holder::Key(key("block.comparator.click"))
}

/// `SoundEvents.COMPOSTER_EMPTY`.
pub fn composter_empty() -> Holder<SoundEvent> {
    Holder::Key(key("block.composter.empty"))
}

/// `SoundEvents.COMPOSTER_FILL`.
pub fn composter_fill() -> Holder<SoundEvent> {
    Holder::Key(key("block.composter.fill"))
}

/// `SoundEvents.COMPOSTER_FILL_SUCCESS`.
pub fn composter_fill_success() -> Holder<SoundEvent> {
    Holder::Key(key("block.composter.fill_success"))
}

/// `SoundEvents.COMPOSTER_READY`.
pub fn composter_ready() -> Holder<SoundEvent> {
    Holder::Key(key("block.composter.ready"))
}

/// `SoundEvents.CONDUIT_ACTIVATE`.
pub fn conduit_activate() -> Holder<SoundEvent> {
    Holder::Key(key("block.conduit.activate"))
}

/// `SoundEvents.CONDUIT_AMBIENT`.
pub fn conduit_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.conduit.ambient"))
}

/// `SoundEvents.CONDUIT_AMBIENT_SHORT`.
pub fn conduit_ambient_short() -> Holder<SoundEvent> {
    Holder::Key(key("block.conduit.ambient.short"))
}

/// `SoundEvents.CONDUIT_ATTACK_TARGET`.
pub fn conduit_attack_target() -> Holder<SoundEvent> {
    Holder::Key(key("block.conduit.attack.target"))
}

/// `SoundEvents.CONDUIT_DEACTIVATE`.
pub fn conduit_deactivate() -> Holder<SoundEvent> {
    Holder::Key(key("block.conduit.deactivate"))
}

/// `SoundEvents.COPPER_BULB_BREAK`.
pub fn copper_bulb_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_bulb.break"))
}

/// `SoundEvents.COPPER_BULB_STEP`.
pub fn copper_bulb_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_bulb.step"))
}

/// `SoundEvents.COPPER_BULB_PLACE`.
pub fn copper_bulb_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_bulb.place"))
}

/// `SoundEvents.COPPER_BULB_HIT`.
pub fn copper_bulb_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_bulb.hit"))
}

/// `SoundEvents.COPPER_BULB_FALL`.
pub fn copper_bulb_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_bulb.fall"))
}

/// `SoundEvents.COPPER_BULB_TURN_ON`.
pub fn copper_bulb_turn_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_bulb.turn_on"))
}

/// `SoundEvents.COPPER_BULB_TURN_OFF`.
pub fn copper_bulb_turn_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_bulb.turn_off"))
}

/// `SoundEvents.COPPER_BREAK`.
pub fn copper_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper.break"))
}

/// `SoundEvents.COPPER_STEP`.
pub fn copper_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper.step"))
}

/// `SoundEvents.COPPER_PLACE`.
pub fn copper_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper.place"))
}

/// `SoundEvents.COPPER_HIT`.
pub fn copper_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper.hit"))
}

/// `SoundEvents.COPPER_FALL`.
pub fn copper_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper.fall"))
}

/// `SoundEvents.COPPER_CHEST_CLOSE`.
pub fn copper_chest_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_chest.close"))
}

/// `SoundEvents.COPPER_CHEST_OPEN`.
pub fn copper_chest_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_chest.open"))
}

/// `SoundEvents.COPPER_CHEST_WEATHERED_CLOSE`.
pub fn copper_chest_weathered_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_chest_weathered.close"))
}

/// `SoundEvents.COPPER_CHEST_WEATHERED_OPEN`.
pub fn copper_chest_weathered_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_chest_weathered.open"))
}

/// `SoundEvents.COPPER_CHEST_OXIDIZED_CLOSE`.
pub fn copper_chest_oxidized_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_chest_oxidized.close"))
}

/// `SoundEvents.COPPER_CHEST_OXIDIZED_OPEN`.
pub fn copper_chest_oxidized_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_chest_oxidized.open"))
}

/// `SoundEvents.COPPER_DOOR_CLOSE`.
pub fn copper_door_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_door.close"))
}

/// `SoundEvents.COPPER_DOOR_OPEN`.
pub fn copper_door_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_door.open"))
}

/// `SoundEvents.COPPER_GOLEM_STEP`.
pub fn copper_golem_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.step"))
}

/// `SoundEvents.COPPER_GOLEM_HURT`.
pub fn copper_golem_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.hurt"))
}

/// `SoundEvents.COPPER_GOLEM_DEATH`.
pub fn copper_golem_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.death"))
}

/// `SoundEvents.COPPER_GOLEM_WEATHERED_STEP`.
pub fn copper_golem_weathered_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_weathered.step"))
}

/// `SoundEvents.COPPER_GOLEM_WEATHERED_HURT`.
pub fn copper_golem_weathered_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_weathered.hurt"))
}

/// `SoundEvents.COPPER_GOLEM_WEATHERED_DEATH`.
pub fn copper_golem_weathered_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_weathered.death"))
}

/// `SoundEvents.COPPER_GOLEM_OXIDIZED_STEP`.
pub fn copper_golem_oxidized_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_oxidized.step"))
}

/// `SoundEvents.COPPER_GOLEM_OXIDIZED_HURT`.
pub fn copper_golem_oxidized_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_oxidized.hurt"))
}

/// `SoundEvents.COPPER_GOLEM_OXIDIZED_DEATH`.
pub fn copper_golem_oxidized_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_oxidized.death"))
}

/// `SoundEvents.COPPER_GOLEM_SPIN`.
pub fn copper_golem_spin() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.spin"))
}

/// `SoundEvents.COPPER_GOLEM_WEATHERED_SPIN`.
pub fn copper_golem_weathered_spin() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_weathered.spin"))
}

/// `SoundEvents.COPPER_GOLEM_OXIDIZED_SPIN`.
pub fn copper_golem_oxidized_spin() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_oxidized.spin"))
}

/// `SoundEvents.COPPER_GOLEM_ITEM_GET`.
pub fn copper_golem_item_get() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.no_item_get"))
}

/// `SoundEvents.COPPER_GOLEM_ITEM_NO_GET`.
pub fn copper_golem_item_no_get() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.no_item_no_get"))
}

/// `SoundEvents.COPPER_GOLEM_ITEM_DROP`.
pub fn copper_golem_item_drop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.item_drop"))
}

/// `SoundEvents.COPPER_GOLEM_ITEM_NO_DROP`.
pub fn copper_golem_item_no_drop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.item_no_drop"))
}

/// `SoundEvents.COPPER_GOLEM_BECOME_STATUE`.
pub fn copper_golem_become_statue() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem_become_statue"))
}

/// `SoundEvents.COPPER_GOLEM_STATUE_BREAK`.
pub fn copper_golem_statue_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_golem_statue.break"))
}

/// `SoundEvents.COPPER_GOLEM_STATUE_PLACE`.
pub fn copper_golem_statue_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_golem_statue.place"))
}

/// `SoundEvents.COPPER_GOLEM_STATUE_HIT`.
pub fn copper_golem_statue_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_golem_statue.hit"))
}

/// `SoundEvents.COPPER_GOLEM_STATUE_STEP`.
pub fn copper_golem_statue_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_golem_statue.step"))
}

/// `SoundEvents.COPPER_GOLEM_STATUE_FALL`.
pub fn copper_golem_statue_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_golem_statue.fall"))
}

/// `SoundEvents.COPPER_GOLEM_SPAWN`.
pub fn copper_golem_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.spawn"))
}

/// `SoundEvents.COPPER_GOLEM_SHEAR`.
pub fn copper_golem_shear() -> Holder<SoundEvent> {
    Holder::Key(key("entity.copper_golem.shear"))
}

/// `SoundEvents.COPPER_GRATE_BREAK`.
pub fn copper_grate_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_grate.break"))
}

/// `SoundEvents.COPPER_GRATE_STEP`.
pub fn copper_grate_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_grate.step"))
}

/// `SoundEvents.COPPER_GRATE_PLACE`.
pub fn copper_grate_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_grate.place"))
}

/// `SoundEvents.COPPER_GRATE_HIT`.
pub fn copper_grate_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_grate.hit"))
}

/// `SoundEvents.COPPER_GRATE_FALL`.
pub fn copper_grate_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_grate.fall"))
}

/// `SoundEvents.COPPER_TRAPDOOR_CLOSE`.
pub fn copper_trapdoor_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_trapdoor.close"))
}

/// `SoundEvents.COPPER_TRAPDOOR_OPEN`.
pub fn copper_trapdoor_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.copper_trapdoor.open"))
}

/// `SoundEvents.CORAL_BLOCK_BREAK`.
pub fn coral_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.coral_block.break"))
}

/// `SoundEvents.CORAL_BLOCK_FALL`.
pub fn coral_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.coral_block.fall"))
}

/// `SoundEvents.CORAL_BLOCK_HIT`.
pub fn coral_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.coral_block.hit"))
}

/// `SoundEvents.CORAL_BLOCK_PLACE`.
pub fn coral_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.coral_block.place"))
}

/// `SoundEvents.CORAL_BLOCK_STEP`.
pub fn coral_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.coral_block.step"))
}

/// `SoundEvents.COW_AMBIENT`.
pub fn cow_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cow.ambient"))
}

/// `SoundEvents.COW_DEATH`.
pub fn cow_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cow.death"))
}

/// `SoundEvents.COW_HURT`.
pub fn cow_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cow.hurt"))
}

/// `SoundEvents.COW_MILK`.
pub fn cow_milk() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cow.milk"))
}

/// `SoundEvents.COW_STEP`.
pub fn cow_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.cow.step"))
}

/// `SoundEvents.CRAFTER_CRAFT`.
pub fn crafter_craft() -> Holder<SoundEvent> {
    Holder::Key(key("block.crafter.craft"))
}

/// `SoundEvents.CRAFTER_FAIL`.
pub fn crafter_fail() -> Holder<SoundEvent> {
    Holder::Key(key("block.crafter.fail"))
}

/// `SoundEvents.CREAKING_AMBIENT`.
pub fn creaking_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.ambient"))
}

/// `SoundEvents.CREAKING_ACTIVATE`.
pub fn creaking_activate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.activate"))
}

/// `SoundEvents.CREAKING_DEACTIVATE`.
pub fn creaking_deactivate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.deactivate"))
}

/// `SoundEvents.CREAKING_ATTACK`.
pub fn creaking_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.attack"))
}

/// `SoundEvents.CREAKING_DEATH`.
pub fn creaking_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.death"))
}

/// `SoundEvents.CREAKING_STEP`.
pub fn creaking_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.step"))
}

/// `SoundEvents.CREAKING_FREEZE`.
pub fn creaking_freeze() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.freeze"))
}

/// `SoundEvents.CREAKING_UNFREEZE`.
pub fn creaking_unfreeze() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.unfreeze"))
}

/// `SoundEvents.CREAKING_SPAWN`.
pub fn creaking_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.spawn"))
}

/// `SoundEvents.CREAKING_SWAY`.
pub fn creaking_sway() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.sway"))
}

/// `SoundEvents.CREAKING_TWITCH`.
pub fn creaking_twitch() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creaking.twitch"))
}

/// `SoundEvents.CREAKING_HEART_BREAK`.
pub fn creaking_heart_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.creaking_heart.break"))
}

/// `SoundEvents.CREAKING_HEART_FALL`.
pub fn creaking_heart_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.creaking_heart.fall"))
}

/// `SoundEvents.CREAKING_HEART_HIT`.
pub fn creaking_heart_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.creaking_heart.hit"))
}

/// `SoundEvents.CREAKING_HEART_HURT`.
pub fn creaking_heart_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("block.creaking_heart.hurt"))
}

/// `SoundEvents.CREAKING_HEART_PLACE`.
pub fn creaking_heart_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.creaking_heart.place"))
}

/// `SoundEvents.CREAKING_HEART_STEP`.
pub fn creaking_heart_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.creaking_heart.step"))
}

/// `SoundEvents.CREAKING_HEART_IDLE`.
pub fn creaking_heart_idle() -> Holder<SoundEvent> {
    Holder::Key(key("block.creaking_heart.idle"))
}

/// `SoundEvents.CREAKING_HEART_SPAWN`.
pub fn creaking_heart_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("block.creaking_heart.spawn"))
}

/// `SoundEvents.CREEPER_DEATH`.
pub fn creeper_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creeper.death"))
}

/// `SoundEvents.CREEPER_HURT`.
pub fn creeper_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creeper.hurt"))
}

/// `SoundEvents.CREEPER_PRIMED`.
pub fn creeper_primed() -> Holder<SoundEvent> {
    Holder::Key(key("entity.creeper.primed"))
}

/// `SoundEvents.CROP_BREAK`.
pub fn crop_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.crop.break"))
}

/// `SoundEvents.CROP_PLANTED`.
pub fn crop_planted() -> Holder<SoundEvent> {
    Holder::Key(key("item.crop.plant"))
}

/// `SoundEvents.CROSSBOW_HIT`.
pub fn crossbow_hit() -> Holder<SoundEvent> {
    Holder::Key(key("item.crossbow.hit"))
}

/// `SoundEvents.CROSSBOW_SHOOT`.
pub fn crossbow_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("item.crossbow.shoot"))
}

/// `SoundEvents.DEAD_BUSH_IDLE`.
pub fn dead_bush_idle() -> Holder<SoundEvent> {
    Holder::Key(key("block.deadbush.idle"))
}

/// `SoundEvents.DECORATED_POT_BREAK`.
pub fn decorated_pot_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.decorated_pot.break"))
}

/// `SoundEvents.DECORATED_POT_FALL`.
pub fn decorated_pot_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.decorated_pot.fall"))
}

/// `SoundEvents.DECORATED_POT_HIT`.
pub fn decorated_pot_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.decorated_pot.hit"))
}

/// `SoundEvents.DECORATED_POT_INSERT`.
pub fn decorated_pot_insert() -> Holder<SoundEvent> {
    Holder::Key(key("block.decorated_pot.insert"))
}

/// `SoundEvents.DECORATED_POT_INSERT_FAIL`.
pub fn decorated_pot_insert_fail() -> Holder<SoundEvent> {
    Holder::Key(key("block.decorated_pot.insert_fail"))
}

/// `SoundEvents.DECORATED_POT_STEP`.
pub fn decorated_pot_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.decorated_pot.step"))
}

/// `SoundEvents.DECORATED_POT_PLACE`.
pub fn decorated_pot_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.decorated_pot.place"))
}

/// `SoundEvents.DECORATED_POT_SHATTER`.
pub fn decorated_pot_shatter() -> Holder<SoundEvent> {
    Holder::Key(key("block.decorated_pot.shatter"))
}

/// `SoundEvents.DEEPSLATE_BRICKS_BREAK`.
pub fn deepslate_bricks_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_bricks.break"))
}

/// `SoundEvents.DEEPSLATE_BRICKS_FALL`.
pub fn deepslate_bricks_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_bricks.fall"))
}

/// `SoundEvents.DEEPSLATE_BRICKS_HIT`.
pub fn deepslate_bricks_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_bricks.hit"))
}

/// `SoundEvents.DEEPSLATE_BRICKS_PLACE`.
pub fn deepslate_bricks_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_bricks.place"))
}

/// `SoundEvents.DEEPSLATE_BRICKS_STEP`.
pub fn deepslate_bricks_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_bricks.step"))
}

/// `SoundEvents.DEEPSLATE_BREAK`.
pub fn deepslate_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate.break"))
}

/// `SoundEvents.DEEPSLATE_FALL`.
pub fn deepslate_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate.fall"))
}

/// `SoundEvents.DEEPSLATE_HIT`.
pub fn deepslate_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate.hit"))
}

/// `SoundEvents.DEEPSLATE_PLACE`.
pub fn deepslate_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate.place"))
}

/// `SoundEvents.DEEPSLATE_STEP`.
pub fn deepslate_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate.step"))
}

/// `SoundEvents.DEEPSLATE_TILES_BREAK`.
pub fn deepslate_tiles_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_tiles.break"))
}

/// `SoundEvents.DEEPSLATE_TILES_FALL`.
pub fn deepslate_tiles_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_tiles.fall"))
}

/// `SoundEvents.DEEPSLATE_TILES_HIT`.
pub fn deepslate_tiles_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_tiles.hit"))
}

/// `SoundEvents.DEEPSLATE_TILES_PLACE`.
pub fn deepslate_tiles_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_tiles.place"))
}

/// `SoundEvents.DEEPSLATE_TILES_STEP`.
pub fn deepslate_tiles_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.deepslate_tiles.step"))
}

/// `SoundEvents.DISPENSER_DISPENSE`.
pub fn dispenser_dispense() -> Holder<SoundEvent> {
    Holder::Key(key("block.dispenser.dispense"))
}

/// `SoundEvents.DISPENSER_FAIL`.
pub fn dispenser_fail() -> Holder<SoundEvent> {
    Holder::Key(key("block.dispenser.fail"))
}

/// `SoundEvents.DISPENSER_LAUNCH`.
pub fn dispenser_launch() -> Holder<SoundEvent> {
    Holder::Key(key("block.dispenser.launch"))
}

/// `SoundEvents.DOLPHIN_AMBIENT`.
pub fn dolphin_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.ambient"))
}

/// `SoundEvents.DOLPHIN_AMBIENT_WATER`.
pub fn dolphin_ambient_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.ambient_water"))
}

/// `SoundEvents.DOLPHIN_ATTACK`.
pub fn dolphin_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.attack"))
}

/// `SoundEvents.DOLPHIN_DEATH`.
pub fn dolphin_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.death"))
}

/// `SoundEvents.DOLPHIN_EAT`.
pub fn dolphin_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.eat"))
}

/// `SoundEvents.DOLPHIN_HURT`.
pub fn dolphin_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.hurt"))
}

/// `SoundEvents.DOLPHIN_JUMP`.
pub fn dolphin_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.jump"))
}

/// `SoundEvents.DOLPHIN_PLAY`.
pub fn dolphin_play() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.play"))
}

/// `SoundEvents.DOLPHIN_SPLASH`.
pub fn dolphin_splash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.splash"))
}

/// `SoundEvents.DOLPHIN_SWIM`.
pub fn dolphin_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dolphin.swim"))
}

/// `SoundEvents.DONKEY_AMBIENT`.
pub fn donkey_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.donkey.ambient"))
}

/// `SoundEvents.DONKEY_ANGRY`.
pub fn donkey_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.donkey.angry"))
}

/// `SoundEvents.DONKEY_CHEST`.
pub fn donkey_chest() -> Holder<SoundEvent> {
    Holder::Key(key("entity.donkey.chest"))
}

/// `SoundEvents.DONKEY_DEATH`.
pub fn donkey_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.donkey.death"))
}

/// `SoundEvents.DONKEY_EAT`.
pub fn donkey_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.donkey.eat"))
}

/// `SoundEvents.DONKEY_HURT`.
pub fn donkey_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.donkey.hurt"))
}

/// `SoundEvents.DONKEY_JUMP`.
pub fn donkey_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.donkey.jump"))
}

/// `SoundEvents.DRIED_GHAST_BREAK`.
pub fn dried_ghast_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.dried_ghast.break"))
}

/// `SoundEvents.DRIED_GHAST_STEP`.
pub fn dried_ghast_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.dried_ghast.step"))
}

/// `SoundEvents.DRIED_GHAST_FALL`.
pub fn dried_ghast_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.dried_ghast.fall"))
}

/// `SoundEvents.DRIED_GHAST_AMBIENT`.
pub fn dried_ghast_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.dried_ghast.ambient"))
}

/// `SoundEvents.DRIED_GHAST_AMBIENT_WATER`.
pub fn dried_ghast_ambient_water() -> Holder<SoundEvent> {
    Holder::Key(key("block.dried_ghast.ambient_water"))
}

/// `SoundEvents.DRIED_GHAST_PLACE`.
pub fn dried_ghast_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.dried_ghast.place"))
}

/// `SoundEvents.DRIED_GHAST_PLACE_IN_WATER`.
pub fn dried_ghast_place_in_water() -> Holder<SoundEvent> {
    Holder::Key(key("block.dried_ghast.place_in_water"))
}

/// `SoundEvents.DRIED_GHAST_TRANSITION`.
pub fn dried_ghast_transition() -> Holder<SoundEvent> {
    Holder::Key(key("block.dried_ghast.transition"))
}

/// `SoundEvents.DRIPSTONE_BLOCK_BREAK`.
pub fn dripstone_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.dripstone_block.break"))
}

/// `SoundEvents.DRIPSTONE_BLOCK_STEP`.
pub fn dripstone_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.dripstone_block.step"))
}

/// `SoundEvents.DRIPSTONE_BLOCK_PLACE`.
pub fn dripstone_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.dripstone_block.place"))
}

/// `SoundEvents.DRIPSTONE_BLOCK_HIT`.
pub fn dripstone_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.dripstone_block.hit"))
}

/// `SoundEvents.DRIPSTONE_BLOCK_FALL`.
pub fn dripstone_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.dripstone_block.fall"))
}

/// `SoundEvents.DRY_GRASS`.
pub fn dry_grass() -> Holder<SoundEvent> {
    Holder::Key(key("block.dry_grass.ambient"))
}

/// `SoundEvents.POINTED_DRIPSTONE_BREAK`.
pub fn pointed_dripstone_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.break"))
}

/// `SoundEvents.POINTED_DRIPSTONE_STEP`.
pub fn pointed_dripstone_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.step"))
}

/// `SoundEvents.POINTED_DRIPSTONE_PLACE`.
pub fn pointed_dripstone_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.place"))
}

/// `SoundEvents.POINTED_DRIPSTONE_HIT`.
pub fn pointed_dripstone_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.hit"))
}

/// `SoundEvents.POINTED_DRIPSTONE_FALL`.
pub fn pointed_dripstone_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.fall"))
}

/// `SoundEvents.POINTED_DRIPSTONE_LAND`.
pub fn pointed_dripstone_land() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.land"))
}

/// `SoundEvents.POINTED_DRIPSTONE_DRIP_LAVA`.
pub fn pointed_dripstone_drip_lava() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.drip_lava"))
}

/// `SoundEvents.POINTED_DRIPSTONE_DRIP_WATER`.
pub fn pointed_dripstone_drip_water() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.drip_water"))
}

/// `SoundEvents.POINTED_DRIPSTONE_DRIP_LAVA_INTO_CAULDRON`.
pub fn pointed_dripstone_drip_lava_into_cauldron() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.drip_lava_into_cauldron"))
}

/// `SoundEvents.POINTED_DRIPSTONE_DRIP_WATER_INTO_CAULDRON`.
pub fn pointed_dripstone_drip_water_into_cauldron() -> Holder<SoundEvent> {
    Holder::Key(key("block.pointed_dripstone.drip_water_into_cauldron"))
}

/// `SoundEvents.BIG_DRIPLEAF_TILT_DOWN`.
pub fn big_dripleaf_tilt_down() -> Holder<SoundEvent> {
    Holder::Key(key("block.big_dripleaf.tilt_down"))
}

/// `SoundEvents.BIG_DRIPLEAF_TILT_UP`.
pub fn big_dripleaf_tilt_up() -> Holder<SoundEvent> {
    Holder::Key(key("block.big_dripleaf.tilt_up"))
}

/// `SoundEvents.DROWNED_AMBIENT`.
pub fn drowned_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.ambient"))
}

/// `SoundEvents.DROWNED_AMBIENT_WATER`.
pub fn drowned_ambient_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.ambient_water"))
}

/// `SoundEvents.DROWNED_DEATH`.
pub fn drowned_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.death"))
}

/// `SoundEvents.DROWNED_DEATH_WATER`.
pub fn drowned_death_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.death_water"))
}

/// `SoundEvents.DROWNED_HURT`.
pub fn drowned_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.hurt"))
}

/// `SoundEvents.DROWNED_HURT_WATER`.
pub fn drowned_hurt_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.hurt_water"))
}

/// `SoundEvents.DROWNED_SHOOT`.
pub fn drowned_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.shoot"))
}

/// `SoundEvents.DROWNED_STEP`.
pub fn drowned_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.step"))
}

/// `SoundEvents.DROWNED_SWIM`.
pub fn drowned_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.drowned.swim"))
}

/// `SoundEvents.DYE_USE`.
pub fn dye_use() -> Holder<SoundEvent> {
    Holder::Key(key("item.dye.use"))
}

/// `SoundEvents.EGG_THROW`.
pub fn egg_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.egg.throw"))
}

/// `SoundEvents.ELDER_GUARDIAN_AMBIENT`.
pub fn elder_guardian_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.elder_guardian.ambient"))
}

/// `SoundEvents.ELDER_GUARDIAN_AMBIENT_LAND`.
pub fn elder_guardian_ambient_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.elder_guardian.ambient_land"))
}

/// `SoundEvents.ELDER_GUARDIAN_CURSE`.
pub fn elder_guardian_curse() -> Holder<SoundEvent> {
    Holder::Key(key("entity.elder_guardian.curse"))
}

/// `SoundEvents.ELDER_GUARDIAN_DEATH`.
pub fn elder_guardian_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.elder_guardian.death"))
}

/// `SoundEvents.ELDER_GUARDIAN_DEATH_LAND`.
pub fn elder_guardian_death_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.elder_guardian.death_land"))
}

/// `SoundEvents.ELDER_GUARDIAN_FLOP`.
pub fn elder_guardian_flop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.elder_guardian.flop"))
}

/// `SoundEvents.ELDER_GUARDIAN_HURT`.
pub fn elder_guardian_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.elder_guardian.hurt"))
}

/// `SoundEvents.ELDER_GUARDIAN_HURT_LAND`.
pub fn elder_guardian_hurt_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.elder_guardian.hurt_land"))
}

/// `SoundEvents.ELYTRA_FLYING`.
pub fn elytra_flying() -> Holder<SoundEvent> {
    Holder::Key(key("item.elytra.flying"))
}

/// `SoundEvents.ENCHANTMENT_TABLE_USE`.
pub fn enchantment_table_use() -> Holder<SoundEvent> {
    Holder::Key(key("block.enchantment_table.use"))
}

/// `SoundEvents.ENDER_CHEST_CLOSE`.
pub fn ender_chest_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.ender_chest.close"))
}

/// `SoundEvents.ENDER_CHEST_OPEN`.
pub fn ender_chest_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.ender_chest.open"))
}

/// `SoundEvents.ENDER_DRAGON_AMBIENT`.
pub fn ender_dragon_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_dragon.ambient"))
}

/// `SoundEvents.ENDER_DRAGON_DEATH`.
pub fn ender_dragon_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_dragon.death"))
}

/// `SoundEvents.DRAGON_FIREBALL_EXPLODE`.
pub fn dragon_fireball_explode() -> Holder<SoundEvent> {
    Holder::Key(key("entity.dragon_fireball.explode"))
}

/// `SoundEvents.ENDER_DRAGON_FLAP`.
pub fn ender_dragon_flap() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_dragon.flap"))
}

/// `SoundEvents.ENDER_DRAGON_GROWL`.
pub fn ender_dragon_growl() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_dragon.growl"))
}

/// `SoundEvents.ENDER_DRAGON_HURT`.
pub fn ender_dragon_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_dragon.hurt"))
}

/// `SoundEvents.ENDER_DRAGON_SHOOT`.
pub fn ender_dragon_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_dragon.shoot"))
}

/// `SoundEvents.ENDER_EYE_DEATH`.
pub fn ender_eye_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_eye.death"))
}

/// `SoundEvents.ENDER_EYE_LAUNCH`.
pub fn ender_eye_launch() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_eye.launch"))
}

/// `SoundEvents.ENDERMAN_AMBIENT`.
pub fn enderman_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.enderman.ambient"))
}

/// `SoundEvents.ENDERMAN_DEATH`.
pub fn enderman_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.enderman.death"))
}

/// `SoundEvents.ENDERMAN_HURT`.
pub fn enderman_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.enderman.hurt"))
}

/// `SoundEvents.ENDERMAN_SCREAM`.
pub fn enderman_scream() -> Holder<SoundEvent> {
    Holder::Key(key("entity.enderman.scream"))
}

/// `SoundEvents.ENDERMAN_STARE`.
pub fn enderman_stare() -> Holder<SoundEvent> {
    Holder::Key(key("entity.enderman.stare"))
}

/// `SoundEvents.ENDERMAN_TELEPORT`.
pub fn enderman_teleport() -> Holder<SoundEvent> {
    Holder::Key(key("entity.enderman.teleport"))
}

/// `SoundEvents.ENDERMITE_AMBIENT`.
pub fn endermite_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.endermite.ambient"))
}

/// `SoundEvents.ENDERMITE_DEATH`.
pub fn endermite_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.endermite.death"))
}

/// `SoundEvents.ENDERMITE_HURT`.
pub fn endermite_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.endermite.hurt"))
}

/// `SoundEvents.ENDERMITE_STEP`.
pub fn endermite_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.endermite.step"))
}

/// `SoundEvents.ENDER_PEARL_THROW`.
pub fn ender_pearl_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ender_pearl.throw"))
}

/// `SoundEvents.END_GATEWAY_SPAWN`.
pub fn end_gateway_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("block.end_gateway.spawn"))
}

/// `SoundEvents.END_PORTAL_FRAME_FILL`.
pub fn end_portal_frame_fill() -> Holder<SoundEvent> {
    Holder::Key(key("block.end_portal_frame.fill"))
}

/// `SoundEvents.END_PORTAL_SPAWN`.
pub fn end_portal_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("block.end_portal.spawn"))
}

/// `SoundEvents.EVOKER_AMBIENT`.
pub fn evoker_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker.ambient"))
}

/// `SoundEvents.EVOKER_CAST_SPELL`.
pub fn evoker_cast_spell() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker.cast_spell"))
}

/// `SoundEvents.EVOKER_CELEBRATE`.
pub fn evoker_celebrate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker.celebrate"))
}

/// `SoundEvents.EVOKER_DEATH`.
pub fn evoker_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker.death"))
}

/// `SoundEvents.EVOKER_FANGS_ATTACK`.
pub fn evoker_fangs_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker_fangs.attack"))
}

/// `SoundEvents.EVOKER_HURT`.
pub fn evoker_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker.hurt"))
}

/// `SoundEvents.EVOKER_PREPARE_ATTACK`.
pub fn evoker_prepare_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker.prepare_attack"))
}

/// `SoundEvents.EVOKER_PREPARE_SUMMON`.
pub fn evoker_prepare_summon() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker.prepare_summon"))
}

/// `SoundEvents.EVOKER_PREPARE_WOLOLO`.
pub fn evoker_prepare_wololo() -> Holder<SoundEvent> {
    Holder::Key(key("entity.evoker.prepare_wololo"))
}

/// `SoundEvents.EXPERIENCE_BOTTLE_THROW`.
pub fn experience_bottle_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.experience_bottle.throw"))
}

/// `SoundEvents.EXPERIENCE_ORB_PICKUP`.
pub fn experience_orb_pickup() -> Holder<SoundEvent> {
    Holder::Key(key("entity.experience_orb.pickup"))
}

/// `SoundEvents.EYEBLOSSOM_OPEN_LONG`.
pub fn eyeblossom_open_long() -> Holder<SoundEvent> {
    Holder::Key(key("block.eyeblossom.open_long"))
}

/// `SoundEvents.EYEBLOSSOM_OPEN`.
pub fn eyeblossom_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.eyeblossom.open"))
}

/// `SoundEvents.EYEBLOSSOM_CLOSE_LONG`.
pub fn eyeblossom_close_long() -> Holder<SoundEvent> {
    Holder::Key(key("block.eyeblossom.close_long"))
}

/// `SoundEvents.EYEBLOSSOM_CLOSE`.
pub fn eyeblossom_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.eyeblossom.close"))
}

/// `SoundEvents.EYEBLOSSOM_IDLE`.
pub fn eyeblossom_idle() -> Holder<SoundEvent> {
    Holder::Key(key("block.eyeblossom.idle"))
}

/// `SoundEvents.FENCE_GATE_CLOSE`.
pub fn fence_gate_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.fence_gate.close"))
}

/// `SoundEvents.FENCE_GATE_OPEN`.
pub fn fence_gate_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.fence_gate.open"))
}

/// `SoundEvents.FIRECHARGE_USE`.
pub fn firecharge_use() -> Holder<SoundEvent> {
    Holder::Key(key("item.firecharge.use"))
}

/// `SoundEvents.FIREFLY_BUSH_IDLE`.
pub fn firefly_bush_idle() -> Holder<SoundEvent> {
    Holder::Key(key("block.firefly_bush.idle"))
}

/// `SoundEvents.FIREWORK_ROCKET_BLAST`.
pub fn firework_rocket_blast() -> Holder<SoundEvent> {
    Holder::Key(key("entity.firework_rocket.blast"))
}

/// `SoundEvents.FIREWORK_ROCKET_BLAST_FAR`.
pub fn firework_rocket_blast_far() -> Holder<SoundEvent> {
    Holder::Key(key("entity.firework_rocket.blast_far"))
}

/// `SoundEvents.FIREWORK_ROCKET_LARGE_BLAST`.
pub fn firework_rocket_large_blast() -> Holder<SoundEvent> {
    Holder::Key(key("entity.firework_rocket.large_blast"))
}

/// `SoundEvents.FIREWORK_ROCKET_LARGE_BLAST_FAR`.
pub fn firework_rocket_large_blast_far() -> Holder<SoundEvent> {
    Holder::Key(key("entity.firework_rocket.large_blast_far"))
}

/// `SoundEvents.FIREWORK_ROCKET_LAUNCH`.
pub fn firework_rocket_launch() -> Holder<SoundEvent> {
    Holder::Key(key("entity.firework_rocket.launch"))
}

/// `SoundEvents.FIREWORK_ROCKET_SHOOT`.
pub fn firework_rocket_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.firework_rocket.shoot"))
}

/// `SoundEvents.FIREWORK_ROCKET_TWINKLE`.
pub fn firework_rocket_twinkle() -> Holder<SoundEvent> {
    Holder::Key(key("entity.firework_rocket.twinkle"))
}

/// `SoundEvents.FIREWORK_ROCKET_TWINKLE_FAR`.
pub fn firework_rocket_twinkle_far() -> Holder<SoundEvent> {
    Holder::Key(key("entity.firework_rocket.twinkle_far"))
}

/// `SoundEvents.FIRE_AMBIENT`.
pub fn fire_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.fire.ambient"))
}

/// `SoundEvents.FIRE_EXTINGUISH`.
pub fn fire_extinguish() -> Holder<SoundEvent> {
    Holder::Key(key("block.fire.extinguish"))
}

/// `SoundEvents.FISH_SWIM`.
pub fn fish_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fish.swim"))
}

/// `SoundEvents.FISHING_BOBBER_RETRIEVE`.
pub fn fishing_bobber_retrieve() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fishing_bobber.retrieve"))
}

/// `SoundEvents.FISHING_BOBBER_SPLASH`.
pub fn fishing_bobber_splash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fishing_bobber.splash"))
}

/// `SoundEvents.FISHING_BOBBER_THROW`.
pub fn fishing_bobber_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fishing_bobber.throw"))
}

/// `SoundEvents.FLINTANDSTEEL_USE`.
pub fn flintandsteel_use() -> Holder<SoundEvent> {
    Holder::Key(key("item.flintandsteel.use"))
}

/// `SoundEvents.FLOWERING_AZALEA_BREAK`.
pub fn flowering_azalea_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.flowering_azalea.break"))
}

/// `SoundEvents.FLOWERING_AZALEA_FALL`.
pub fn flowering_azalea_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.flowering_azalea.fall"))
}

/// `SoundEvents.FLOWERING_AZALEA_HIT`.
pub fn flowering_azalea_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.flowering_azalea.hit"))
}

/// `SoundEvents.FLOWERING_AZALEA_PLACE`.
pub fn flowering_azalea_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.flowering_azalea.place"))
}

/// `SoundEvents.FLOWERING_AZALEA_STEP`.
pub fn flowering_azalea_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.flowering_azalea.step"))
}

/// `SoundEvents.FOX_AGGRO`.
pub fn fox_aggro() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.aggro"))
}

/// `SoundEvents.FOX_AMBIENT`.
pub fn fox_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.ambient"))
}

/// `SoundEvents.FOX_BITE`.
pub fn fox_bite() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.bite"))
}

/// `SoundEvents.FOX_DEATH`.
pub fn fox_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.death"))
}

/// `SoundEvents.FOX_EAT`.
pub fn fox_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.eat"))
}

/// `SoundEvents.FOX_HURT`.
pub fn fox_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.hurt"))
}

/// `SoundEvents.FOX_SCREECH`.
pub fn fox_screech() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.screech"))
}

/// `SoundEvents.FOX_SLEEP`.
pub fn fox_sleep() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.sleep"))
}

/// `SoundEvents.FOX_SNIFF`.
pub fn fox_sniff() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.sniff"))
}

/// `SoundEvents.FOX_SPIT`.
pub fn fox_spit() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.spit"))
}

/// `SoundEvents.FOX_TELEPORT`.
pub fn fox_teleport() -> Holder<SoundEvent> {
    Holder::Key(key("entity.fox.teleport"))
}

/// `SoundEvents.SUSPICIOUS_SAND_BREAK`.
pub fn suspicious_sand_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_sand.break"))
}

/// `SoundEvents.SUSPICIOUS_SAND_STEP`.
pub fn suspicious_sand_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_sand.step"))
}

/// `SoundEvents.SUSPICIOUS_SAND_PLACE`.
pub fn suspicious_sand_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_sand.place"))
}

/// `SoundEvents.SUSPICIOUS_SAND_HIT`.
pub fn suspicious_sand_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_sand.hit"))
}

/// `SoundEvents.SUSPICIOUS_SAND_FALL`.
pub fn suspicious_sand_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_sand.fall"))
}

/// `SoundEvents.SUSPICIOUS_GRAVEL_BREAK`.
pub fn suspicious_gravel_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_gravel.break"))
}

/// `SoundEvents.SUSPICIOUS_GRAVEL_STEP`.
pub fn suspicious_gravel_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_gravel.step"))
}

/// `SoundEvents.SUSPICIOUS_GRAVEL_PLACE`.
pub fn suspicious_gravel_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_gravel.place"))
}

/// `SoundEvents.SUSPICIOUS_GRAVEL_HIT`.
pub fn suspicious_gravel_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_gravel.hit"))
}

/// `SoundEvents.SUSPICIOUS_GRAVEL_FALL`.
pub fn suspicious_gravel_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.suspicious_gravel.fall"))
}

/// `SoundEvents.FROGLIGHT_BREAK`.
pub fn froglight_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.froglight.break"))
}

/// `SoundEvents.FROGLIGHT_FALL`.
pub fn froglight_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.froglight.fall"))
}

/// `SoundEvents.FROGLIGHT_HIT`.
pub fn froglight_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.froglight.hit"))
}

/// `SoundEvents.FROGLIGHT_PLACE`.
pub fn froglight_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.froglight.place"))
}

/// `SoundEvents.FROGLIGHT_STEP`.
pub fn froglight_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.froglight.step"))
}

/// `SoundEvents.FROGSPAWNSTEP`.
pub fn frogspawnstep() -> Holder<SoundEvent> {
    Holder::Key(key("block.frogspawn.step"))
}

/// `SoundEvents.FROGSPAWN_BREAK`.
pub fn frogspawn_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.frogspawn.break"))
}

/// `SoundEvents.FROGSPAWN_FALL`.
pub fn frogspawn_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.frogspawn.fall"))
}

/// `SoundEvents.FROGSPAWN_HATCH`.
pub fn frogspawn_hatch() -> Holder<SoundEvent> {
    Holder::Key(key("block.frogspawn.hatch"))
}

/// `SoundEvents.FROGSPAWN_HIT`.
pub fn frogspawn_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.frogspawn.hit"))
}

/// `SoundEvents.FROGSPAWN_PLACE`.
pub fn frogspawn_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.frogspawn.place"))
}

/// `SoundEvents.FROG_AMBIENT`.
pub fn frog_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.frog.ambient"))
}

/// `SoundEvents.FROG_DEATH`.
pub fn frog_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.frog.death"))
}

/// `SoundEvents.FROG_EAT`.
pub fn frog_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.frog.eat"))
}

/// `SoundEvents.FROG_HURT`.
pub fn frog_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.frog.hurt"))
}

/// `SoundEvents.FROG_LAY_SPAWN`.
pub fn frog_lay_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("entity.frog.lay_spawn"))
}

/// `SoundEvents.FROG_LONG_JUMP`.
pub fn frog_long_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.frog.long_jump"))
}

/// `SoundEvents.FROG_STEP`.
pub fn frog_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.frog.step"))
}

/// `SoundEvents.FROG_TONGUE`.
pub fn frog_tongue() -> Holder<SoundEvent> {
    Holder::Key(key("entity.frog.tongue"))
}

/// `SoundEvents.ROOTS_BREAK`.
pub fn roots_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.roots.break"))
}

/// `SoundEvents.ROOTS_STEP`.
pub fn roots_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.roots.step"))
}

/// `SoundEvents.ROOTS_PLACE`.
pub fn roots_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.roots.place"))
}

/// `SoundEvents.ROOTS_HIT`.
pub fn roots_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.roots.hit"))
}

/// `SoundEvents.ROOTS_FALL`.
pub fn roots_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.roots.fall"))
}

/// `SoundEvents.FURNACE_FIRE_CRACKLE`.
pub fn furnace_fire_crackle() -> Holder<SoundEvent> {
    Holder::Key(key("block.furnace.fire_crackle"))
}

/// `SoundEvents.GENERIC_BIG_FALL`.
pub fn generic_big_fall() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.big_fall"))
}

/// `SoundEvents.GENERIC_BURN`.
pub fn generic_burn() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.burn"))
}

/// `SoundEvents.GENERIC_DEATH`.
pub fn generic_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.death"))
}

/// `SoundEvents.GENERIC_DRINK`.
pub fn generic_drink() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.drink"))
}

/// `SoundEvents.GENERIC_EAT`.
pub fn generic_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.eat"))
}

/// `SoundEvents.GENERIC_EXPLODE`.
pub fn generic_explode() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.explode"))
}

/// `SoundEvents.GENERIC_EXTINGUISH_FIRE`.
pub fn generic_extinguish_fire() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.extinguish_fire"))
}

/// `SoundEvents.GENERIC_HURT`.
pub fn generic_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.hurt"))
}

/// `SoundEvents.GENERIC_SMALL_FALL`.
pub fn generic_small_fall() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.small_fall"))
}

/// `SoundEvents.GENERIC_SPLASH`.
pub fn generic_splash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.splash"))
}

/// `SoundEvents.GENERIC_SWIM`.
pub fn generic_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.generic.swim"))
}

/// `SoundEvents.GHAST_AMBIENT`.
pub fn ghast_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghast.ambient"))
}

/// `SoundEvents.GHAST_DEATH`.
pub fn ghast_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghast.death"))
}

/// `SoundEvents.GHAST_HURT`.
pub fn ghast_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghast.hurt"))
}

/// `SoundEvents.GHAST_SCREAM`.
pub fn ghast_scream() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghast.scream"))
}

/// `SoundEvents.GHAST_SHOOT`.
pub fn ghast_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghast.shoot"))
}

/// `SoundEvents.GHAST_WARN`.
pub fn ghast_warn() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghast.warn"))
}

/// `SoundEvents.GHASTLING_AMBIENT`.
pub fn ghastling_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghastling.ambient"))
}

/// `SoundEvents.GHASTLING_DEATH`.
pub fn ghastling_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghastling.death"))
}

/// `SoundEvents.GHASTLING_HURT`.
pub fn ghastling_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghastling.hurt"))
}

/// `SoundEvents.GHASTLING_SPAWN`.
pub fn ghastling_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ghastling.spawn"))
}

/// `SoundEvents.GILDED_BLACKSTONE_BREAK`.
pub fn gilded_blackstone_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.gilded_blackstone.break"))
}

/// `SoundEvents.GILDED_BLACKSTONE_FALL`.
pub fn gilded_blackstone_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.gilded_blackstone.fall"))
}

/// `SoundEvents.GILDED_BLACKSTONE_HIT`.
pub fn gilded_blackstone_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.gilded_blackstone.hit"))
}

/// `SoundEvents.GILDED_BLACKSTONE_PLACE`.
pub fn gilded_blackstone_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.gilded_blackstone.place"))
}

/// `SoundEvents.GILDED_BLACKSTONE_STEP`.
pub fn gilded_blackstone_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.gilded_blackstone.step"))
}

/// `SoundEvents.GLASS_BREAK`.
pub fn glass_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.glass.break"))
}

/// `SoundEvents.GLASS_FALL`.
pub fn glass_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.glass.fall"))
}

/// `SoundEvents.GLASS_HIT`.
pub fn glass_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.glass.hit"))
}

/// `SoundEvents.GLASS_PLACE`.
pub fn glass_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.glass.place"))
}

/// `SoundEvents.GLASS_STEP`.
pub fn glass_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.glass.step"))
}

/// `SoundEvents.GLOW_INK_SAC_USE`.
pub fn glow_ink_sac_use() -> Holder<SoundEvent> {
    Holder::Key(key("item.glow_ink_sac.use"))
}

/// `SoundEvents.GLOW_ITEM_FRAME_ADD_ITEM`.
pub fn glow_item_frame_add_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_item_frame.add_item"))
}

/// `SoundEvents.GLOW_ITEM_FRAME_BREAK`.
pub fn glow_item_frame_break() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_item_frame.break"))
}

/// `SoundEvents.GLOW_ITEM_FRAME_PLACE`.
pub fn glow_item_frame_place() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_item_frame.place"))
}

/// `SoundEvents.GLOW_ITEM_FRAME_REMOVE_ITEM`.
pub fn glow_item_frame_remove_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_item_frame.remove_item"))
}

/// `SoundEvents.GLOW_ITEM_FRAME_ROTATE_ITEM`.
pub fn glow_item_frame_rotate_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_item_frame.rotate_item"))
}

/// `SoundEvents.GLOW_SQUID_AMBIENT`.
pub fn glow_squid_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_squid.ambient"))
}

/// `SoundEvents.GLOW_SQUID_DEATH`.
pub fn glow_squid_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_squid.death"))
}

/// `SoundEvents.GLOW_SQUID_HURT`.
pub fn glow_squid_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_squid.hurt"))
}

/// `SoundEvents.GLOW_SQUID_SQUIRT`.
pub fn glow_squid_squirt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.glow_squid.squirt"))
}

/// `SoundEvents.GOAT_AMBIENT`.
pub fn goat_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.ambient"))
}

/// `SoundEvents.GOAT_DEATH`.
pub fn goat_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.death"))
}

/// `SoundEvents.GOAT_EAT`.
pub fn goat_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.eat"))
}

/// `SoundEvents.GOAT_HURT`.
pub fn goat_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.hurt"))
}

/// `SoundEvents.GOAT_LONG_JUMP`.
pub fn goat_long_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.long_jump"))
}

/// `SoundEvents.GOAT_MILK`.
pub fn goat_milk() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.milk"))
}

/// `SoundEvents.GOAT_PREPARE_RAM`.
pub fn goat_prepare_ram() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.prepare_ram"))
}

/// `SoundEvents.GOAT_RAM_IMPACT`.
pub fn goat_ram_impact() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.ram_impact"))
}

/// `SoundEvents.GOAT_HORN_BREAK`.
pub fn goat_horn_break() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.horn_break"))
}

/// `SoundEvents.GOAT_SCREAMING_AMBIENT`.
pub fn goat_screaming_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.screaming.ambient"))
}

/// `SoundEvents.GOAT_SCREAMING_DEATH`.
pub fn goat_screaming_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.screaming.death"))
}

/// `SoundEvents.GOAT_SCREAMING_EAT`.
pub fn goat_screaming_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.screaming.eat"))
}

/// `SoundEvents.GOAT_SCREAMING_HURT`.
pub fn goat_screaming_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.screaming.hurt"))
}

/// `SoundEvents.GOAT_SCREAMING_LONG_JUMP`.
pub fn goat_screaming_long_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.screaming.long_jump"))
}

/// `SoundEvents.GOAT_SCREAMING_MILK`.
pub fn goat_screaming_milk() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.screaming.milk"))
}

/// `SoundEvents.GOAT_SCREAMING_PREPARE_RAM`.
pub fn goat_screaming_prepare_ram() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.screaming.prepare_ram"))
}

/// `SoundEvents.GOAT_SCREAMING_RAM_IMPACT`.
pub fn goat_screaming_ram_impact() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.screaming.ram_impact"))
}

/// `SoundEvents.GOAT_STEP`.
pub fn goat_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.goat.step"))
}

/// `SoundEvents.GRASS_BREAK`.
pub fn grass_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.grass.break"))
}

/// `SoundEvents.GRASS_FALL`.
pub fn grass_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.grass.fall"))
}

/// `SoundEvents.GRASS_HIT`.
pub fn grass_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.grass.hit"))
}

/// `SoundEvents.GRASS_PLACE`.
pub fn grass_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.grass.place"))
}

/// `SoundEvents.GRASS_STEP`.
pub fn grass_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.grass.step"))
}

/// `SoundEvents.GRAVEL_BREAK`.
pub fn gravel_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.gravel.break"))
}

/// `SoundEvents.GRAVEL_FALL`.
pub fn gravel_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.gravel.fall"))
}

/// `SoundEvents.GRAVEL_HIT`.
pub fn gravel_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.gravel.hit"))
}

/// `SoundEvents.GRAVEL_PLACE`.
pub fn gravel_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.gravel.place"))
}

/// `SoundEvents.GRAVEL_STEP`.
pub fn gravel_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.gravel.step"))
}

/// `SoundEvents.GRINDSTONE_USE`.
pub fn grindstone_use() -> Holder<SoundEvent> {
    Holder::Key(key("block.grindstone.use"))
}

/// `SoundEvents.GROWING_PLANT_CROP`.
pub fn growing_plant_crop() -> Holder<SoundEvent> {
    Holder::Key(key("block.growing_plant.crop"))
}

/// `SoundEvents.GUARDIAN_AMBIENT`.
pub fn guardian_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.guardian.ambient"))
}

/// `SoundEvents.GUARDIAN_AMBIENT_LAND`.
pub fn guardian_ambient_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.guardian.ambient_land"))
}

/// `SoundEvents.GUARDIAN_ATTACK`.
pub fn guardian_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.guardian.attack"))
}

/// `SoundEvents.GUARDIAN_DEATH`.
pub fn guardian_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.guardian.death"))
}

/// `SoundEvents.GUARDIAN_DEATH_LAND`.
pub fn guardian_death_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.guardian.death_land"))
}

/// `SoundEvents.GUARDIAN_FLOP`.
pub fn guardian_flop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.guardian.flop"))
}

/// `SoundEvents.GUARDIAN_HURT`.
pub fn guardian_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.guardian.hurt"))
}

/// `SoundEvents.GUARDIAN_HURT_LAND`.
pub fn guardian_hurt_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.guardian.hurt_land"))
}

/// `SoundEvents.HANGING_ROOTS_BREAK`.
pub fn hanging_roots_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_roots.break"))
}

/// `SoundEvents.HANGING_ROOTS_FALL`.
pub fn hanging_roots_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_roots.fall"))
}

/// `SoundEvents.HANGING_ROOTS_HIT`.
pub fn hanging_roots_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_roots.hit"))
}

/// `SoundEvents.HANGING_ROOTS_PLACE`.
pub fn hanging_roots_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_roots.place"))
}

/// `SoundEvents.HANGING_ROOTS_STEP`.
pub fn hanging_roots_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_roots.step"))
}

/// `SoundEvents.HANGING_SIGN_STEP`.
pub fn hanging_sign_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_sign.step"))
}

/// `SoundEvents.HANGING_SIGN_BREAK`.
pub fn hanging_sign_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_sign.break"))
}

/// `SoundEvents.HANGING_SIGN_FALL`.
pub fn hanging_sign_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_sign.fall"))
}

/// `SoundEvents.HANGING_SIGN_HIT`.
pub fn hanging_sign_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_sign.hit"))
}

/// `SoundEvents.HANGING_SIGN_PLACE`.
pub fn hanging_sign_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_sign.place"))
}

/// `SoundEvents.HAPPY_GHAST_AMBIENT`.
pub fn happy_ghast_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.happy_ghast.ambient"))
}

/// `SoundEvents.HAPPY_GHAST_DEATH`.
pub fn happy_ghast_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.happy_ghast.death"))
}

/// `SoundEvents.HAPPY_GHAST_HURT`.
pub fn happy_ghast_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.happy_ghast.hurt"))
}

/// `SoundEvents.HAPPY_GHAST_RIDING`.
pub fn happy_ghast_riding() -> Holder<SoundEvent> {
    Holder::Key(key("entity.happy_ghast.riding"))
}

/// `SoundEvents.HEAVY_CORE_BREAK`.
pub fn heavy_core_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.heavy_core.break"))
}

/// `SoundEvents.HEAVY_CORE_FALL`.
pub fn heavy_core_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.heavy_core.fall"))
}

/// `SoundEvents.HEAVY_CORE_HIT`.
pub fn heavy_core_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.heavy_core.hit"))
}

/// `SoundEvents.HEAVY_CORE_PLACE`.
pub fn heavy_core_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.heavy_core.place"))
}

/// `SoundEvents.HEAVY_CORE_STEP`.
pub fn heavy_core_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.heavy_core.step"))
}

/// `SoundEvents.NETHER_WOOD_HANGING_SIGN_STEP`.
pub fn nether_wood_hanging_sign_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_hanging_sign.step"))
}

/// `SoundEvents.NETHER_WOOD_HANGING_SIGN_BREAK`.
pub fn nether_wood_hanging_sign_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_hanging_sign.break"))
}

/// `SoundEvents.NETHER_WOOD_HANGING_SIGN_FALL`.
pub fn nether_wood_hanging_sign_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_hanging_sign.fall"))
}

/// `SoundEvents.NETHER_WOOD_HANGING_SIGN_HIT`.
pub fn nether_wood_hanging_sign_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_hanging_sign.hit"))
}

/// `SoundEvents.NETHER_WOOD_HANGING_SIGN_PLACE`.
pub fn nether_wood_hanging_sign_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_hanging_sign.place"))
}

/// `SoundEvents.BAMBOO_WOOD_HANGING_SIGN_STEP`.
pub fn bamboo_wood_hanging_sign_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_hanging_sign.step"))
}

/// `SoundEvents.BAMBOO_WOOD_HANGING_SIGN_BREAK`.
pub fn bamboo_wood_hanging_sign_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_hanging_sign.break"))
}

/// `SoundEvents.BAMBOO_WOOD_HANGING_SIGN_FALL`.
pub fn bamboo_wood_hanging_sign_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_hanging_sign.fall"))
}

/// `SoundEvents.BAMBOO_WOOD_HANGING_SIGN_HIT`.
pub fn bamboo_wood_hanging_sign_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_hanging_sign.hit"))
}

/// `SoundEvents.BAMBOO_WOOD_HANGING_SIGN_PLACE`.
pub fn bamboo_wood_hanging_sign_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.bamboo_wood_hanging_sign.place"))
}

/// `SoundEvents.TRIAL_SPAWNER_BREAK`.
pub fn trial_spawner_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.break"))
}

/// `SoundEvents.TRIAL_SPAWNER_STEP`.
pub fn trial_spawner_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.step"))
}

/// `SoundEvents.TRIAL_SPAWNER_PLACE`.
pub fn trial_spawner_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.place"))
}

/// `SoundEvents.TRIAL_SPAWNER_HIT`.
pub fn trial_spawner_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.hit"))
}

/// `SoundEvents.TRIAL_SPAWNER_FALL`.
pub fn trial_spawner_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.fall"))
}

/// `SoundEvents.TRIAL_SPAWNER_SPAWN_MOB`.
pub fn trial_spawner_spawn_mob() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.spawn_mob"))
}

/// `SoundEvents.TRIAL_SPAWNER_ABOUT_TO_SPAWN_ITEM`.
pub fn trial_spawner_about_to_spawn_item() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.about_to_spawn_item"))
}

/// `SoundEvents.TRIAL_SPAWNER_SPAWN_ITEM`.
pub fn trial_spawner_spawn_item() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.spawn_item"))
}

/// `SoundEvents.TRIAL_SPAWNER_SPAWN_ITEM_BEGIN`.
pub fn trial_spawner_spawn_item_begin() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.spawn_item_begin"))
}

/// `SoundEvents.TRIAL_SPAWNER_DETECT_PLAYER`.
pub fn trial_spawner_detect_player() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.detect_player"))
}

/// `SoundEvents.TRIAL_SPAWNER_OMINOUS_ACTIVATE`.
pub fn trial_spawner_ominous_activate() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.ominous_activate"))
}

/// `SoundEvents.TRIAL_SPAWNER_AMBIENT`.
pub fn trial_spawner_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.ambient"))
}

/// `SoundEvents.TRIAL_SPAWNER_AMBIENT_OMINOUS`.
pub fn trial_spawner_ambient_ominous() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.ambient_ominous"))
}

/// `SoundEvents.TRIAL_SPAWNER_OPEN_SHUTTER`.
pub fn trial_spawner_open_shutter() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.open_shutter"))
}

/// `SoundEvents.TRIAL_SPAWNER_CLOSE_SHUTTER`.
pub fn trial_spawner_close_shutter() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.close_shutter"))
}

/// `SoundEvents.TRIAL_SPAWNER_EJECT_ITEM`.
pub fn trial_spawner_eject_item() -> Holder<SoundEvent> {
    Holder::Key(key("block.trial_spawner.eject_item"))
}

/// `SoundEvents.HARNESS_EQUIP`.
pub fn harness_equip() -> Holder<SoundEvent> {
    Holder::Key(key("entity.happy_ghast.equip"))
}

/// `SoundEvents.HARNESS_UNEQUIP`.
pub fn harness_unequip() -> Holder<SoundEvent> {
    Holder::Key(key("entity.happy_ghast.unequip"))
}

/// `SoundEvents.HARNESS_GOGGLES_UP`.
pub fn harness_goggles_up() -> Holder<SoundEvent> {
    Holder::Key(key("entity.happy_ghast.harness_goggles_up"))
}

/// `SoundEvents.HARNESS_GOGGLES_DOWN`.
pub fn harness_goggles_down() -> Holder<SoundEvent> {
    Holder::Key(key("entity.happy_ghast.harness_goggles_down"))
}

/// `SoundEvents.HOE_TILL`.
pub fn hoe_till() -> Holder<SoundEvent> {
    Holder::Key(key("item.hoe.till"))
}

/// `SoundEvents.HOGLIN_AMBIENT`.
pub fn hoglin_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hoglin.ambient"))
}

/// `SoundEvents.HOGLIN_ANGRY`.
pub fn hoglin_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hoglin.angry"))
}

/// `SoundEvents.HOGLIN_ATTACK`.
pub fn hoglin_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hoglin.attack"))
}

/// `SoundEvents.HOGLIN_CONVERTED_TO_ZOMBIFIED`.
pub fn hoglin_converted_to_zombified() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hoglin.converted_to_zombified"))
}

/// `SoundEvents.HOGLIN_DEATH`.
pub fn hoglin_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hoglin.death"))
}

/// `SoundEvents.HOGLIN_HURT`.
pub fn hoglin_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hoglin.hurt"))
}

/// `SoundEvents.HOGLIN_RETREAT`.
pub fn hoglin_retreat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hoglin.retreat"))
}

/// `SoundEvents.HOGLIN_STEP`.
pub fn hoglin_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hoglin.step"))
}

/// `SoundEvents.HONEY_BLOCK_BREAK`.
pub fn honey_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.honey_block.break"))
}

/// `SoundEvents.HONEY_BLOCK_FALL`.
pub fn honey_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.honey_block.fall"))
}

/// `SoundEvents.HONEY_BLOCK_HIT`.
pub fn honey_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.honey_block.hit"))
}

/// `SoundEvents.HONEY_BLOCK_PLACE`.
pub fn honey_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.honey_block.place"))
}

/// `SoundEvents.HONEY_BLOCK_SLIDE`.
pub fn honey_block_slide() -> Holder<SoundEvent> {
    Holder::Key(key("block.honey_block.slide"))
}

/// `SoundEvents.HONEY_BLOCK_STEP`.
pub fn honey_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.honey_block.step"))
}

/// `SoundEvents.HONEYCOMB_WAX_ON`.
pub fn honeycomb_wax_on() -> Holder<SoundEvent> {
    Holder::Key(key("item.honeycomb.wax_on"))
}

/// `SoundEvents.HONEY_DRINK`.
pub fn honey_drink() -> Holder<SoundEvent> {
    Holder::Key(key("item.honey_bottle.drink"))
}

/// `SoundEvents.HORSE_AMBIENT`.
pub fn horse_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.ambient"))
}

/// `SoundEvents.HORSE_ANGRY`.
pub fn horse_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.angry"))
}

/// `SoundEvents.HORSE_ARMOR`.
pub fn horse_armor() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.armor"))
}

/// `SoundEvents.HORSE_ARMOR_UNEQUIP`.
pub fn horse_armor_unequip() -> Holder<SoundEvent> {
    Holder::Key(key("item.horse_armor.unequip"))
}

/// `SoundEvents.HORSE_BREATHE`.
pub fn horse_breathe() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.breathe"))
}

/// `SoundEvents.HORSE_DEATH`.
pub fn horse_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.death"))
}

/// `SoundEvents.HORSE_EAT`.
pub fn horse_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.eat"))
}

/// `SoundEvents.HORSE_GALLOP`.
pub fn horse_gallop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.gallop"))
}

/// `SoundEvents.HORSE_HURT`.
pub fn horse_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.hurt"))
}

/// `SoundEvents.HORSE_JUMP`.
pub fn horse_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.jump"))
}

/// `SoundEvents.HORSE_LAND`.
pub fn horse_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.land"))
}

/// `SoundEvents.HORSE_SADDLE`.
pub fn horse_saddle() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.saddle"))
}

/// `SoundEvents.HORSE_STEP`.
pub fn horse_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.step"))
}

/// `SoundEvents.HORSE_STEP_WOOD`.
pub fn horse_step_wood() -> Holder<SoundEvent> {
    Holder::Key(key("entity.horse.step_wood"))
}

/// `SoundEvents.HOSTILE_BIG_FALL`.
pub fn hostile_big_fall() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hostile.big_fall"))
}

/// `SoundEvents.HOSTILE_DEATH`.
pub fn hostile_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hostile.death"))
}

/// `SoundEvents.HOSTILE_HURT`.
pub fn hostile_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hostile.hurt"))
}

/// `SoundEvents.HOSTILE_SMALL_FALL`.
pub fn hostile_small_fall() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hostile.small_fall"))
}

/// `SoundEvents.HOSTILE_SPLASH`.
pub fn hostile_splash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hostile.splash"))
}

/// `SoundEvents.HOSTILE_SWIM`.
pub fn hostile_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.hostile.swim"))
}

/// `SoundEvents.HUSK_AMBIENT`.
pub fn husk_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.husk.ambient"))
}

/// `SoundEvents.HUSK_CONVERTED_TO_ZOMBIE`.
pub fn husk_converted_to_zombie() -> Holder<SoundEvent> {
    Holder::Key(key("entity.husk.converted_to_zombie"))
}

/// `SoundEvents.HUSK_DEATH`.
pub fn husk_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.husk.death"))
}

/// `SoundEvents.HUSK_HURT`.
pub fn husk_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.husk.hurt"))
}

/// `SoundEvents.HUSK_STEP`.
pub fn husk_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.husk.step"))
}

/// `SoundEvents.ILLUSIONER_AMBIENT`.
pub fn illusioner_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.illusioner.ambient"))
}

/// `SoundEvents.ILLUSIONER_CAST_SPELL`.
pub fn illusioner_cast_spell() -> Holder<SoundEvent> {
    Holder::Key(key("entity.illusioner.cast_spell"))
}

/// `SoundEvents.ILLUSIONER_DEATH`.
pub fn illusioner_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.illusioner.death"))
}

/// `SoundEvents.ILLUSIONER_HURT`.
pub fn illusioner_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.illusioner.hurt"))
}

/// `SoundEvents.ILLUSIONER_MIRROR_MOVE`.
pub fn illusioner_mirror_move() -> Holder<SoundEvent> {
    Holder::Key(key("entity.illusioner.mirror_move"))
}

/// `SoundEvents.ILLUSIONER_PREPARE_BLINDNESS`.
pub fn illusioner_prepare_blindness() -> Holder<SoundEvent> {
    Holder::Key(key("entity.illusioner.prepare_blindness"))
}

/// `SoundEvents.ILLUSIONER_PREPARE_MIRROR`.
pub fn illusioner_prepare_mirror() -> Holder<SoundEvent> {
    Holder::Key(key("entity.illusioner.prepare_mirror"))
}

/// `SoundEvents.INK_SAC_USE`.
pub fn ink_sac_use() -> Holder<SoundEvent> {
    Holder::Key(key("item.ink_sac.use"))
}

/// `SoundEvents.IRON_BREAK`.
pub fn iron_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron.break"))
}

/// `SoundEvents.IRON_STEP`.
pub fn iron_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron.step"))
}

/// `SoundEvents.IRON_PLACE`.
pub fn iron_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron.place"))
}

/// `SoundEvents.IRON_HIT`.
pub fn iron_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron.hit"))
}

/// `SoundEvents.IRON_FALL`.
pub fn iron_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron.fall"))
}

/// `SoundEvents.IRON_DOOR_CLOSE`.
pub fn iron_door_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron_door.close"))
}

/// `SoundEvents.IRON_DOOR_OPEN`.
pub fn iron_door_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron_door.open"))
}

/// `SoundEvents.IRON_GOLEM_ATTACK`.
pub fn iron_golem_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.iron_golem.attack"))
}

/// `SoundEvents.IRON_GOLEM_DAMAGE`.
pub fn iron_golem_damage() -> Holder<SoundEvent> {
    Holder::Key(key("entity.iron_golem.damage"))
}

/// `SoundEvents.IRON_GOLEM_DEATH`.
pub fn iron_golem_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.iron_golem.death"))
}

/// `SoundEvents.IRON_GOLEM_HURT`.
pub fn iron_golem_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.iron_golem.hurt"))
}

/// `SoundEvents.IRON_GOLEM_REPAIR`.
pub fn iron_golem_repair() -> Holder<SoundEvent> {
    Holder::Key(key("entity.iron_golem.repair"))
}

/// `SoundEvents.IRON_GOLEM_STEP`.
pub fn iron_golem_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.iron_golem.step"))
}

/// `SoundEvents.IRON_TRAPDOOR_CLOSE`.
pub fn iron_trapdoor_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron_trapdoor.close"))
}

/// `SoundEvents.IRON_TRAPDOOR_OPEN`.
pub fn iron_trapdoor_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.iron_trapdoor.open"))
}

/// `SoundEvents.ITEM_FRAME_ADD_ITEM`.
pub fn item_frame_add_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.item_frame.add_item"))
}

/// `SoundEvents.ITEM_FRAME_BREAK`.
pub fn item_frame_break() -> Holder<SoundEvent> {
    Holder::Key(key("entity.item_frame.break"))
}

/// `SoundEvents.ITEM_FRAME_PLACE`.
pub fn item_frame_place() -> Holder<SoundEvent> {
    Holder::Key(key("entity.item_frame.place"))
}

/// `SoundEvents.ITEM_FRAME_REMOVE_ITEM`.
pub fn item_frame_remove_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.item_frame.remove_item"))
}

/// `SoundEvents.ITEM_FRAME_ROTATE_ITEM`.
pub fn item_frame_rotate_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.item_frame.rotate_item"))
}

/// `SoundEvents.ITEM_BREAK`.
pub fn item_break() -> Holder<SoundEvent> {
    Holder::Key(key("entity.item.break"))
}

/// `SoundEvents.ITEM_PICKUP`.
pub fn item_pickup() -> Holder<SoundEvent> {
    Holder::Key(key("entity.item.pickup"))
}

/// `SoundEvents.LADDER_BREAK`.
pub fn ladder_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.ladder.break"))
}

/// `SoundEvents.LADDER_FALL`.
pub fn ladder_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.ladder.fall"))
}

/// `SoundEvents.LADDER_HIT`.
pub fn ladder_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.ladder.hit"))
}

/// `SoundEvents.LADDER_PLACE`.
pub fn ladder_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.ladder.place"))
}

/// `SoundEvents.LADDER_STEP`.
pub fn ladder_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.ladder.step"))
}

/// `SoundEvents.LANTERN_BREAK`.
pub fn lantern_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.lantern.break"))
}

/// `SoundEvents.LANTERN_FALL`.
pub fn lantern_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.lantern.fall"))
}

/// `SoundEvents.LANTERN_HIT`.
pub fn lantern_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.lantern.hit"))
}

/// `SoundEvents.LANTERN_PLACE`.
pub fn lantern_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.lantern.place"))
}

/// `SoundEvents.LANTERN_STEP`.
pub fn lantern_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.lantern.step"))
}

/// `SoundEvents.LARGE_AMETHYST_BUD_BREAK`.
pub fn large_amethyst_bud_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.large_amethyst_bud.break"))
}

/// `SoundEvents.LARGE_AMETHYST_BUD_PLACE`.
pub fn large_amethyst_bud_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.large_amethyst_bud.place"))
}

/// `SoundEvents.LAVA_AMBIENT`.
pub fn lava_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.lava.ambient"))
}

/// `SoundEvents.LAVA_EXTINGUISH`.
pub fn lava_extinguish() -> Holder<SoundEvent> {
    Holder::Key(key("block.lava.extinguish"))
}

/// `SoundEvents.LAVA_POP`.
pub fn lava_pop() -> Holder<SoundEvent> {
    Holder::Key(key("block.lava.pop"))
}

/// `SoundEvents.LEAF_LITTER_BREAK`.
pub fn leaf_litter_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.leaf_litter.break"))
}

/// `SoundEvents.LEAF_LITTER_STEP`.
pub fn leaf_litter_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.leaf_litter.step"))
}

/// `SoundEvents.LEAF_LITTER_PLACE`.
pub fn leaf_litter_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.leaf_litter.place"))
}

/// `SoundEvents.LEAF_LITTER_HIT`.
pub fn leaf_litter_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.leaf_litter.hit"))
}

/// `SoundEvents.LEAF_LITTER_FALL`.
pub fn leaf_litter_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.leaf_litter.fall"))
}

/// `SoundEvents.LEAD_UNTIED`.
pub fn lead_untied() -> Holder<SoundEvent> {
    Holder::Key(key("item.lead.untied"))
}

/// `SoundEvents.LEAD_TIED`.
pub fn lead_tied() -> Holder<SoundEvent> {
    Holder::Key(key("item.lead.tied"))
}

/// `SoundEvents.LEAD_BREAK`.
pub fn lead_break() -> Holder<SoundEvent> {
    Holder::Key(key("item.lead.break"))
}

/// `SoundEvents.LEVER_CLICK`.
pub fn lever_click() -> Holder<SoundEvent> {
    Holder::Key(key("block.lever.click"))
}

/// `SoundEvents.LIGHTNING_BOLT_IMPACT`.
pub fn lightning_bolt_impact() -> Holder<SoundEvent> {
    Holder::Key(key("entity.lightning_bolt.impact"))
}

/// `SoundEvents.LIGHTNING_BOLT_THUNDER`.
pub fn lightning_bolt_thunder() -> Holder<SoundEvent> {
    Holder::Key(key("entity.lightning_bolt.thunder"))
}

/// `SoundEvents.LINGERING_POTION_THROW`.
pub fn lingering_potion_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.lingering_potion.throw"))
}

/// `SoundEvents.LLAMA_AMBIENT`.
pub fn llama_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.ambient"))
}

/// `SoundEvents.LLAMA_ANGRY`.
pub fn llama_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.angry"))
}

/// `SoundEvents.LLAMA_CHEST`.
pub fn llama_chest() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.chest"))
}

/// `SoundEvents.LLAMA_DEATH`.
pub fn llama_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.death"))
}

/// `SoundEvents.LLAMA_EAT`.
pub fn llama_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.eat"))
}

/// `SoundEvents.LLAMA_HURT`.
pub fn llama_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.hurt"))
}

/// `SoundEvents.LLAMA_SPIT`.
pub fn llama_spit() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.spit"))
}

/// `SoundEvents.LLAMA_STEP`.
pub fn llama_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.step"))
}

/// `SoundEvents.LLAMA_SWAG`.
pub fn llama_swag() -> Holder<SoundEvent> {
    Holder::Key(key("entity.llama.swag"))
}

/// `SoundEvents.LLAMA_CARPET_UNEQUIP`.
pub fn llama_carpet_unequip() -> Holder<SoundEvent> {
    Holder::Key(key("item.llama_carpet.unequip"))
}

/// `SoundEvents.MAGMA_CUBE_DEATH_SMALL`.
pub fn magma_cube_death_small() -> Holder<SoundEvent> {
    Holder::Key(key("entity.magma_cube.death_small"))
}

/// `SoundEvents.LODESTONE_BREAK`.
pub fn lodestone_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.lodestone.break"))
}

/// `SoundEvents.LODESTONE_STEP`.
pub fn lodestone_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.lodestone.step"))
}

/// `SoundEvents.LODESTONE_PLACE`.
pub fn lodestone_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.lodestone.place"))
}

/// `SoundEvents.LODESTONE_HIT`.
pub fn lodestone_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.lodestone.hit"))
}

/// `SoundEvents.LODESTONE_FALL`.
pub fn lodestone_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.lodestone.fall"))
}

/// `SoundEvents.LODESTONE_COMPASS_LOCK`.
pub fn lodestone_compass_lock() -> Holder<SoundEvent> {
    Holder::Key(key("item.lodestone_compass.lock"))
}

/// `SoundEvents.MACE_SMASH_AIR`.
pub fn mace_smash_air() -> Holder<SoundEvent> {
    Holder::Key(key("item.mace.smash_air"))
}

/// `SoundEvents.MACE_SMASH_GROUND`.
pub fn mace_smash_ground() -> Holder<SoundEvent> {
    Holder::Key(key("item.mace.smash_ground"))
}

/// `SoundEvents.MACE_SMASH_GROUND_HEAVY`.
pub fn mace_smash_ground_heavy() -> Holder<SoundEvent> {
    Holder::Key(key("item.mace.smash_ground_heavy"))
}

/// `SoundEvents.MAGMA_CUBE_DEATH`.
pub fn magma_cube_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.magma_cube.death"))
}

/// `SoundEvents.MAGMA_CUBE_HURT`.
pub fn magma_cube_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.magma_cube.hurt"))
}

/// `SoundEvents.MAGMA_CUBE_HURT_SMALL`.
pub fn magma_cube_hurt_small() -> Holder<SoundEvent> {
    Holder::Key(key("entity.magma_cube.hurt_small"))
}

/// `SoundEvents.MAGMA_CUBE_JUMP`.
pub fn magma_cube_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.magma_cube.jump"))
}

/// `SoundEvents.MAGMA_CUBE_SQUISH`.
pub fn magma_cube_squish() -> Holder<SoundEvent> {
    Holder::Key(key("entity.magma_cube.squish"))
}

/// `SoundEvents.MAGMA_CUBE_SQUISH_SMALL`.
pub fn magma_cube_squish_small() -> Holder<SoundEvent> {
    Holder::Key(key("entity.magma_cube.squish_small"))
}

/// `SoundEvents.MANGROVE_ROOTS_BREAK`.
pub fn mangrove_roots_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.mangrove_roots.break"))
}

/// `SoundEvents.MANGROVE_ROOTS_FALL`.
pub fn mangrove_roots_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.mangrove_roots.fall"))
}

/// `SoundEvents.MANGROVE_ROOTS_HIT`.
pub fn mangrove_roots_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.mangrove_roots.hit"))
}

/// `SoundEvents.MANGROVE_ROOTS_PLACE`.
pub fn mangrove_roots_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.mangrove_roots.place"))
}

/// `SoundEvents.MANGROVE_ROOTS_STEP`.
pub fn mangrove_roots_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.mangrove_roots.step"))
}

/// `SoundEvents.MEDIUM_AMETHYST_BUD_BREAK`.
pub fn medium_amethyst_bud_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.medium_amethyst_bud.break"))
}

/// `SoundEvents.MEDIUM_AMETHYST_BUD_PLACE`.
pub fn medium_amethyst_bud_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.medium_amethyst_bud.place"))
}

/// `SoundEvents.METAL_BREAK`.
pub fn metal_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.metal.break"))
}

/// `SoundEvents.METAL_FALL`.
pub fn metal_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.metal.fall"))
}

/// `SoundEvents.METAL_HIT`.
pub fn metal_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.metal.hit"))
}

/// `SoundEvents.METAL_PLACE`.
pub fn metal_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.metal.place"))
}

/// `SoundEvents.METAL_PRESSURE_PLATE_CLICK_OFF`.
pub fn metal_pressure_plate_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.metal_pressure_plate.click_off"))
}

/// `SoundEvents.METAL_PRESSURE_PLATE_CLICK_ON`.
pub fn metal_pressure_plate_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.metal_pressure_plate.click_on"))
}

/// `SoundEvents.METAL_STEP`.
pub fn metal_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.metal.step"))
}

/// `SoundEvents.MINECART_INSIDE_UNDERWATER`.
pub fn minecart_inside_underwater() -> Holder<SoundEvent> {
    Holder::Key(key("entity.minecart.inside.underwater"))
}

/// `SoundEvents.MINECART_INSIDE`.
pub fn minecart_inside() -> Holder<SoundEvent> {
    Holder::Key(key("entity.minecart.inside"))
}

/// `SoundEvents.MINECART_RIDING`.
pub fn minecart_riding() -> Holder<SoundEvent> {
    Holder::Key(key("entity.minecart.riding"))
}

/// `SoundEvents.MOOSHROOM_CONVERT`.
pub fn mooshroom_convert() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mooshroom.convert"))
}

/// `SoundEvents.MOOSHROOM_EAT`.
pub fn mooshroom_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mooshroom.eat"))
}

/// `SoundEvents.MOOSHROOM_MILK`.
pub fn mooshroom_milk() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mooshroom.milk"))
}

/// `SoundEvents.MOOSHROOM_MILK_SUSPICIOUSLY`.
pub fn mooshroom_milk_suspiciously() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mooshroom.suspicious_milk"))
}

/// `SoundEvents.MOOSHROOM_SHEAR`.
pub fn mooshroom_shear() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mooshroom.shear"))
}

/// `SoundEvents.MOSS_CARPET_BREAK`.
pub fn moss_carpet_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss_carpet.break"))
}

/// `SoundEvents.MOSS_CARPET_FALL`.
pub fn moss_carpet_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss_carpet.fall"))
}

/// `SoundEvents.MOSS_CARPET_HIT`.
pub fn moss_carpet_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss_carpet.hit"))
}

/// `SoundEvents.MOSS_CARPET_PLACE`.
pub fn moss_carpet_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss_carpet.place"))
}

/// `SoundEvents.MOSS_CARPET_STEP`.
pub fn moss_carpet_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss_carpet.step"))
}

/// `SoundEvents.PINK_PETALS_BREAK`.
pub fn pink_petals_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.pink_petals.break"))
}

/// `SoundEvents.PINK_PETALS_FALL`.
pub fn pink_petals_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.pink_petals.fall"))
}

/// `SoundEvents.PINK_PETALS_HIT`.
pub fn pink_petals_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.pink_petals.hit"))
}

/// `SoundEvents.PINK_PETALS_PLACE`.
pub fn pink_petals_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.pink_petals.place"))
}

/// `SoundEvents.PINK_PETALS_STEP`.
pub fn pink_petals_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.pink_petals.step"))
}

/// `SoundEvents.MOSS_BREAK`.
pub fn moss_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss.break"))
}

/// `SoundEvents.MOSS_FALL`.
pub fn moss_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss.fall"))
}

/// `SoundEvents.MOSS_HIT`.
pub fn moss_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss.hit"))
}

/// `SoundEvents.MOSS_PLACE`.
pub fn moss_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss.place"))
}

/// `SoundEvents.MOSS_STEP`.
pub fn moss_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.moss.step"))
}

/// `SoundEvents.MUD_BREAK`.
pub fn mud_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud.break"))
}

/// `SoundEvents.MUD_FALL`.
pub fn mud_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud.fall"))
}

/// `SoundEvents.MUD_HIT`.
pub fn mud_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud.hit"))
}

/// `SoundEvents.MUD_PLACE`.
pub fn mud_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud.place"))
}

/// `SoundEvents.MUD_STEP`.
pub fn mud_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud.step"))
}

/// `SoundEvents.MUD_BRICKS_BREAK`.
pub fn mud_bricks_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud_bricks.break"))
}

/// `SoundEvents.MUD_BRICKS_FALL`.
pub fn mud_bricks_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud_bricks.fall"))
}

/// `SoundEvents.MUD_BRICKS_HIT`.
pub fn mud_bricks_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud_bricks.hit"))
}

/// `SoundEvents.MUD_BRICKS_PLACE`.
pub fn mud_bricks_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud_bricks.place"))
}

/// `SoundEvents.MUD_BRICKS_STEP`.
pub fn mud_bricks_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.mud_bricks.step"))
}

/// `SoundEvents.MUDDY_MANGROVE_ROOTS_BREAK`.
pub fn muddy_mangrove_roots_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.muddy_mangrove_roots.break"))
}

/// `SoundEvents.MUDDY_MANGROVE_ROOTS_FALL`.
pub fn muddy_mangrove_roots_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.muddy_mangrove_roots.fall"))
}

/// `SoundEvents.MUDDY_MANGROVE_ROOTS_HIT`.
pub fn muddy_mangrove_roots_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.muddy_mangrove_roots.hit"))
}

/// `SoundEvents.MUDDY_MANGROVE_ROOTS_PLACE`.
pub fn muddy_mangrove_roots_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.muddy_mangrove_roots.place"))
}

/// `SoundEvents.MUDDY_MANGROVE_ROOTS_STEP`.
pub fn muddy_mangrove_roots_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.muddy_mangrove_roots.step"))
}

/// `SoundEvents.MULE_AMBIENT`.
pub fn mule_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mule.ambient"))
}

/// `SoundEvents.MULE_ANGRY`.
pub fn mule_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mule.angry"))
}

/// `SoundEvents.MULE_CHEST`.
pub fn mule_chest() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mule.chest"))
}

/// `SoundEvents.MULE_DEATH`.
pub fn mule_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mule.death"))
}

/// `SoundEvents.MULE_EAT`.
pub fn mule_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mule.eat"))
}

/// `SoundEvents.MULE_HURT`.
pub fn mule_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mule.hurt"))
}

/// `SoundEvents.MULE_JUMP`.
pub fn mule_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.mule.jump"))
}

/// `SoundEvents.MUSIC_CREATIVE`.
pub fn music_creative() -> Holder<SoundEvent> {
    Holder::Key(key("music.creative"))
}

/// `SoundEvents.MUSIC_CREDITS`.
pub fn music_credits() -> Holder<SoundEvent> {
    Holder::Key(key("music.credits"))
}

/// `SoundEvents.MUSIC_DISC_5`.
pub fn music_disc_5() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.5"))
}

/// `SoundEvents.MUSIC_DISC_11`.
pub fn music_disc_11() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.11"))
}

/// `SoundEvents.MUSIC_DISC_13`.
pub fn music_disc_13() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.13"))
}

/// `SoundEvents.MUSIC_DISC_BLOCKS`.
pub fn music_disc_blocks() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.blocks"))
}

/// `SoundEvents.MUSIC_DISC_CAT`.
pub fn music_disc_cat() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.cat"))
}

/// `SoundEvents.MUSIC_DISC_CHIRP`.
pub fn music_disc_chirp() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.chirp"))
}

/// `SoundEvents.MUSIC_DISC_FAR`.
pub fn music_disc_far() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.far"))
}

/// `SoundEvents.MUSIC_DISC_LAVA_CHICKEN`.
pub fn music_disc_lava_chicken() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.lava_chicken"))
}

/// `SoundEvents.MUSIC_DISC_MALL`.
pub fn music_disc_mall() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.mall"))
}

/// `SoundEvents.MUSIC_DISC_MELLOHI`.
pub fn music_disc_mellohi() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.mellohi"))
}

/// `SoundEvents.MUSIC_DISC_PIGSTEP`.
pub fn music_disc_pigstep() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.pigstep"))
}

/// `SoundEvents.MUSIC_DISC_STAL`.
pub fn music_disc_stal() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.stal"))
}

/// `SoundEvents.MUSIC_DISC_STRAD`.
pub fn music_disc_strad() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.strad"))
}

/// `SoundEvents.MUSIC_DISC_WAIT`.
pub fn music_disc_wait() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.wait"))
}

/// `SoundEvents.MUSIC_DISC_WARD`.
pub fn music_disc_ward() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.ward"))
}

/// `SoundEvents.MUSIC_DISC_OTHERSIDE`.
pub fn music_disc_otherside() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.otherside"))
}

/// `SoundEvents.MUSIC_DISC_RELIC`.
pub fn music_disc_relic() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.relic"))
}

/// `SoundEvents.MUSIC_DISC_CREATOR`.
pub fn music_disc_creator() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.creator"))
}

/// `SoundEvents.MUSIC_DISC_CREATOR_MUSIC_BOX`.
pub fn music_disc_creator_music_box() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.creator_music_box"))
}

/// `SoundEvents.MUSIC_DISC_PRECIPICE`.
pub fn music_disc_precipice() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.precipice"))
}

/// `SoundEvents.MUSIC_DISC_TEARS`.
pub fn music_disc_tears() -> Holder<SoundEvent> {
    Holder::Key(key("music_disc.tears"))
}

/// `SoundEvents.MUSIC_DRAGON`.
pub fn music_dragon() -> Holder<SoundEvent> {
    Holder::Key(key("music.dragon"))
}

/// `SoundEvents.MUSIC_END`.
pub fn music_end() -> Holder<SoundEvent> {
    Holder::Key(key("music.end"))
}

/// `SoundEvents.MUSIC_GAME`.
pub fn music_game() -> Holder<SoundEvent> {
    Holder::Key(key("music.game"))
}

/// `SoundEvents.MUSIC_MENU`.
pub fn music_menu() -> Holder<SoundEvent> {
    Holder::Key(key("music.menu"))
}

/// `SoundEvents.MUSIC_BIOME_BASALT_DELTAS`.
pub fn music_biome_basalt_deltas() -> Holder<SoundEvent> {
    Holder::Key(key("music.nether.basalt_deltas"))
}

/// `SoundEvents.MUSIC_BIOME_CRIMSON_FOREST`.
pub fn music_biome_crimson_forest() -> Holder<SoundEvent> {
    Holder::Key(key("music.nether.crimson_forest"))
}

/// `SoundEvents.MUSIC_BIOME_DEEP_DARK`.
pub fn music_biome_deep_dark() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.deep_dark"))
}

/// `SoundEvents.MUSIC_BIOME_DRIPSTONE_CAVES`.
pub fn music_biome_dripstone_caves() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.dripstone_caves"))
}

/// `SoundEvents.MUSIC_BIOME_GROVE`.
pub fn music_biome_grove() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.grove"))
}

/// `SoundEvents.MUSIC_BIOME_JAGGED_PEAKS`.
pub fn music_biome_jagged_peaks() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.jagged_peaks"))
}

/// `SoundEvents.MUSIC_BIOME_LUSH_CAVES`.
pub fn music_biome_lush_caves() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.lush_caves"))
}

/// `SoundEvents.MUSIC_BIOME_SWAMP`.
pub fn music_biome_swamp() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.swamp"))
}

/// `SoundEvents.MUSIC_BIOME_FOREST`.
pub fn music_biome_forest() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.forest"))
}

/// `SoundEvents.MUSIC_BIOME_OLD_GROWTH_TAIGA`.
pub fn music_biome_old_growth_taiga() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.old_growth_taiga"))
}

/// `SoundEvents.MUSIC_BIOME_MEADOW`.
pub fn music_biome_meadow() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.meadow"))
}

/// `SoundEvents.MUSIC_BIOME_CHERRY_GROVE`.
pub fn music_biome_cherry_grove() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.cherry_grove"))
}

/// `SoundEvents.MUSIC_BIOME_NETHER_WASTES`.
pub fn music_biome_nether_wastes() -> Holder<SoundEvent> {
    Holder::Key(key("music.nether.nether_wastes"))
}

/// `SoundEvents.MUSIC_BIOME_FROZEN_PEAKS`.
pub fn music_biome_frozen_peaks() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.frozen_peaks"))
}

/// `SoundEvents.MUSIC_BIOME_SNOWY_SLOPES`.
pub fn music_biome_snowy_slopes() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.snowy_slopes"))
}

/// `SoundEvents.MUSIC_BIOME_SOUL_SAND_VALLEY`.
pub fn music_biome_soul_sand_valley() -> Holder<SoundEvent> {
    Holder::Key(key("music.nether.soul_sand_valley"))
}

/// `SoundEvents.MUSIC_BIOME_STONY_PEAKS`.
pub fn music_biome_stony_peaks() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.stony_peaks"))
}

/// `SoundEvents.MUSIC_BIOME_WARPED_FOREST`.
pub fn music_biome_warped_forest() -> Holder<SoundEvent> {
    Holder::Key(key("music.nether.warped_forest"))
}

/// `SoundEvents.MUSIC_BIOME_FLOWER_FOREST`.
pub fn music_biome_flower_forest() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.flower_forest"))
}

/// `SoundEvents.MUSIC_BIOME_DESERT`.
pub fn music_biome_desert() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.desert"))
}

/// `SoundEvents.MUSIC_BIOME_BADLANDS`.
pub fn music_biome_badlands() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.badlands"))
}

/// `SoundEvents.MUSIC_BIOME_JUNGLE`.
pub fn music_biome_jungle() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.jungle"))
}

/// `SoundEvents.MUSIC_BIOME_SPARSE_JUNGLE`.
pub fn music_biome_sparse_jungle() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.sparse_jungle"))
}

/// `SoundEvents.MUSIC_BIOME_BAMBOO_JUNGLE`.
pub fn music_biome_bamboo_jungle() -> Holder<SoundEvent> {
    Holder::Key(key("music.overworld.bamboo_jungle"))
}

/// `SoundEvents.MUSIC_UNDER_WATER`.
pub fn music_under_water() -> Holder<SoundEvent> {
    Holder::Key(key("music.under_water"))
}

/// `SoundEvents.NAUTILUS_AMBIENT`.
pub fn nautilus_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.ambient"))
}

/// `SoundEvents.NAUTILUS_AMBIENT_ON_LAND`.
pub fn nautilus_ambient_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.ambient_land"))
}

/// `SoundEvents.NAUTILUS_DASH`.
pub fn nautilus_dash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.dash"))
}

/// `SoundEvents.NAUTILUS_DASH_ON_LAND`.
pub fn nautilus_dash_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.dash_land"))
}

/// `SoundEvents.NAUTILUS_DASH_READY`.
pub fn nautilus_dash_ready() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.dash_ready"))
}

/// `SoundEvents.NAUTILUS_DASH_READY_ON_LAND`.
pub fn nautilus_dash_ready_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.dash_ready_land"))
}

/// `SoundEvents.NAUTILUS_DEATH`.
pub fn nautilus_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.death"))
}

/// `SoundEvents.NAUTILUS_DEATH_ON_LAND`.
pub fn nautilus_death_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.death_land"))
}

/// `SoundEvents.NAUTILUS_EAT`.
pub fn nautilus_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.eat"))
}

/// `SoundEvents.NAUTILUS_HURT`.
pub fn nautilus_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.hurt"))
}

/// `SoundEvents.NAUTILUS_HURT_ON_LAND`.
pub fn nautilus_hurt_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.hurt_land"))
}

/// `SoundEvents.NAUTILUS_SWIM`.
pub fn nautilus_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.nautilus.swim"))
}

/// `SoundEvents.NETHER_BRICKS_BREAK`.
pub fn nether_bricks_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_bricks.break"))
}

/// `SoundEvents.NETHER_BRICKS_STEP`.
pub fn nether_bricks_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_bricks.step"))
}

/// `SoundEvents.NETHER_BRICKS_PLACE`.
pub fn nether_bricks_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_bricks.place"))
}

/// `SoundEvents.NETHER_BRICKS_HIT`.
pub fn nether_bricks_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_bricks.hit"))
}

/// `SoundEvents.NETHER_BRICKS_FALL`.
pub fn nether_bricks_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_bricks.fall"))
}

/// `SoundEvents.NETHER_WART_BREAK`.
pub fn nether_wart_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wart.break"))
}

/// `SoundEvents.NETHER_WART_PLANTED`.
pub fn nether_wart_planted() -> Holder<SoundEvent> {
    Holder::Key(key("item.nether_wart.plant"))
}

/// `SoundEvents.NETHER_WOOD_BREAK`.
pub fn nether_wood_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood.break"))
}

/// `SoundEvents.NETHER_WOOD_FALL`.
pub fn nether_wood_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood.fall"))
}

/// `SoundEvents.NETHER_WOOD_HIT`.
pub fn nether_wood_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood.hit"))
}

/// `SoundEvents.NETHER_WOOD_PLACE`.
pub fn nether_wood_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood.place"))
}

/// `SoundEvents.NETHER_WOOD_STEP`.
pub fn nether_wood_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood.step"))
}

/// `SoundEvents.NETHER_WOOD_DOOR_CLOSE`.
pub fn nether_wood_door_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_door.close"))
}

/// `SoundEvents.NETHER_WOOD_DOOR_OPEN`.
pub fn nether_wood_door_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_door.open"))
}

/// `SoundEvents.NETHER_WOOD_TRAPDOOR_CLOSE`.
pub fn nether_wood_trapdoor_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_trapdoor.close"))
}

/// `SoundEvents.NETHER_WOOD_TRAPDOOR_OPEN`.
pub fn nether_wood_trapdoor_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_trapdoor.open"))
}

/// `SoundEvents.NETHER_WOOD_BUTTON_CLICK_OFF`.
pub fn nether_wood_button_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_button.click_off"))
}

/// `SoundEvents.NETHER_WOOD_BUTTON_CLICK_ON`.
pub fn nether_wood_button_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_button.click_on"))
}

/// `SoundEvents.NETHER_WOOD_PRESSURE_PLATE_CLICK_OFF`.
pub fn nether_wood_pressure_plate_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_pressure_plate.click_off"))
}

/// `SoundEvents.NETHER_WOOD_PRESSURE_PLATE_CLICK_ON`.
pub fn nether_wood_pressure_plate_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_pressure_plate.click_on"))
}

/// `SoundEvents.NETHER_WOOD_FENCE_GATE_CLOSE`.
pub fn nether_wood_fence_gate_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_fence_gate.close"))
}

/// `SoundEvents.NETHER_WOOD_FENCE_GATE_OPEN`.
pub fn nether_wood_fence_gate_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_wood_fence_gate.open"))
}

/// `SoundEvents.EMPTY`.
pub fn empty() -> Holder<SoundEvent> {
    Holder::Key(key("intentionally_empty"))
}

/// `SoundEvents.PACKED_MUD_BREAK`.
pub fn packed_mud_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.packed_mud.break"))
}

/// `SoundEvents.PACKED_MUD_FALL`.
pub fn packed_mud_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.packed_mud.fall"))
}

/// `SoundEvents.PACKED_MUD_HIT`.
pub fn packed_mud_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.packed_mud.hit"))
}

/// `SoundEvents.PACKED_MUD_PLACE`.
pub fn packed_mud_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.packed_mud.place"))
}

/// `SoundEvents.PACKED_MUD_STEP`.
pub fn packed_mud_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.packed_mud.step"))
}

/// `SoundEvents.STEM_BREAK`.
pub fn stem_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.stem.break"))
}

/// `SoundEvents.STEM_STEP`.
pub fn stem_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.stem.step"))
}

/// `SoundEvents.STEM_PLACE`.
pub fn stem_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.stem.place"))
}

/// `SoundEvents.STEM_HIT`.
pub fn stem_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.stem.hit"))
}

/// `SoundEvents.STEM_FALL`.
pub fn stem_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.stem.fall"))
}

/// `SoundEvents.NYLIUM_BREAK`.
pub fn nylium_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.nylium.break"))
}

/// `SoundEvents.NYLIUM_STEP`.
pub fn nylium_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.nylium.step"))
}

/// `SoundEvents.NYLIUM_PLACE`.
pub fn nylium_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.nylium.place"))
}

/// `SoundEvents.NYLIUM_HIT`.
pub fn nylium_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.nylium.hit"))
}

/// `SoundEvents.NYLIUM_FALL`.
pub fn nylium_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.nylium.fall"))
}

/// `SoundEvents.NETHER_SPROUTS_BREAK`.
pub fn nether_sprouts_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_sprouts.break"))
}

/// `SoundEvents.NETHER_SPROUTS_STEP`.
pub fn nether_sprouts_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_sprouts.step"))
}

/// `SoundEvents.NETHER_SPROUTS_PLACE`.
pub fn nether_sprouts_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_sprouts.place"))
}

/// `SoundEvents.NETHER_SPROUTS_HIT`.
pub fn nether_sprouts_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_sprouts.hit"))
}

/// `SoundEvents.NETHER_SPROUTS_FALL`.
pub fn nether_sprouts_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_sprouts.fall"))
}

/// `SoundEvents.FUNGUS_BREAK`.
pub fn fungus_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.fungus.break"))
}

/// `SoundEvents.FUNGUS_STEP`.
pub fn fungus_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.fungus.step"))
}

/// `SoundEvents.FUNGUS_PLACE`.
pub fn fungus_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.fungus.place"))
}

/// `SoundEvents.FUNGUS_HIT`.
pub fn fungus_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.fungus.hit"))
}

/// `SoundEvents.FUNGUS_FALL`.
pub fn fungus_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.fungus.fall"))
}

/// `SoundEvents.WEEPING_VINES_BREAK`.
pub fn weeping_vines_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.weeping_vines.break"))
}

/// `SoundEvents.WEEPING_VINES_STEP`.
pub fn weeping_vines_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.weeping_vines.step"))
}

/// `SoundEvents.WEEPING_VINES_PLACE`.
pub fn weeping_vines_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.weeping_vines.place"))
}

/// `SoundEvents.WEEPING_VINES_HIT`.
pub fn weeping_vines_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.weeping_vines.hit"))
}

/// `SoundEvents.WEEPING_VINES_FALL`.
pub fn weeping_vines_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.weeping_vines.fall"))
}

/// `SoundEvents.WART_BLOCK_BREAK`.
pub fn wart_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.wart_block.break"))
}

/// `SoundEvents.WART_BLOCK_STEP`.
pub fn wart_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.wart_block.step"))
}

/// `SoundEvents.WART_BLOCK_PLACE`.
pub fn wart_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.wart_block.place"))
}

/// `SoundEvents.WART_BLOCK_HIT`.
pub fn wart_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.wart_block.hit"))
}

/// `SoundEvents.WART_BLOCK_FALL`.
pub fn wart_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.wart_block.fall"))
}

/// `SoundEvents.NETHERITE_BLOCK_BREAK`.
pub fn netherite_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherite_block.break"))
}

/// `SoundEvents.NETHERITE_BLOCK_STEP`.
pub fn netherite_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherite_block.step"))
}

/// `SoundEvents.NETHERITE_BLOCK_PLACE`.
pub fn netherite_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherite_block.place"))
}

/// `SoundEvents.NETHERITE_BLOCK_HIT`.
pub fn netherite_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherite_block.hit"))
}

/// `SoundEvents.NETHERITE_BLOCK_FALL`.
pub fn netherite_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherite_block.fall"))
}

/// `SoundEvents.NETHERRACK_BREAK`.
pub fn netherrack_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherrack.break"))
}

/// `SoundEvents.NETHERRACK_STEP`.
pub fn netherrack_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherrack.step"))
}

/// `SoundEvents.NETHERRACK_PLACE`.
pub fn netherrack_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherrack.place"))
}

/// `SoundEvents.NETHERRACK_HIT`.
pub fn netherrack_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherrack.hit"))
}

/// `SoundEvents.NETHERRACK_FALL`.
pub fn netherrack_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.netherrack.fall"))
}

/// `SoundEvents.NOTE_BLOCK_BASEDRUM`.
pub fn note_block_basedrum() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.basedrum"))
}

/// `SoundEvents.NOTE_BLOCK_BASS`.
pub fn note_block_bass() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.bass"))
}

/// `SoundEvents.NOTE_BLOCK_BELL`.
pub fn note_block_bell() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.bell"))
}

/// `SoundEvents.NOTE_BLOCK_CHIME`.
pub fn note_block_chime() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.chime"))
}

/// `SoundEvents.NOTE_BLOCK_FLUTE`.
pub fn note_block_flute() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.flute"))
}

/// `SoundEvents.NOTE_BLOCK_GUITAR`.
pub fn note_block_guitar() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.guitar"))
}

/// `SoundEvents.NOTE_BLOCK_HARP`.
pub fn note_block_harp() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.harp"))
}

/// `SoundEvents.NOTE_BLOCK_HAT`.
pub fn note_block_hat() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.hat"))
}

/// `SoundEvents.NOTE_BLOCK_PLING`.
pub fn note_block_pling() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.pling"))
}

/// `SoundEvents.NOTE_BLOCK_SNARE`.
pub fn note_block_snare() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.snare"))
}

/// `SoundEvents.NOTE_BLOCK_XYLOPHONE`.
pub fn note_block_xylophone() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.xylophone"))
}

/// `SoundEvents.NOTE_BLOCK_IRON_XYLOPHONE`.
pub fn note_block_iron_xylophone() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.iron_xylophone"))
}

/// `SoundEvents.NOTE_BLOCK_COW_BELL`.
pub fn note_block_cow_bell() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.cow_bell"))
}

/// `SoundEvents.NOTE_BLOCK_DIDGERIDOO`.
pub fn note_block_didgeridoo() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.didgeridoo"))
}

/// `SoundEvents.NOTE_BLOCK_BIT`.
pub fn note_block_bit() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.bit"))
}

/// `SoundEvents.NOTE_BLOCK_BANJO`.
pub fn note_block_banjo() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.banjo"))
}

/// `SoundEvents.NOTE_BLOCK_IMITATE_ZOMBIE`.
pub fn note_block_imitate_zombie() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.imitate.zombie"))
}

/// `SoundEvents.NOTE_BLOCK_IMITATE_SKELETON`.
pub fn note_block_imitate_skeleton() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.imitate.skeleton"))
}

/// `SoundEvents.NOTE_BLOCK_IMITATE_CREEPER`.
pub fn note_block_imitate_creeper() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.imitate.creeper"))
}

/// `SoundEvents.NOTE_BLOCK_IMITATE_ENDER_DRAGON`.
pub fn note_block_imitate_ender_dragon() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.imitate.ender_dragon"))
}

/// `SoundEvents.NOTE_BLOCK_IMITATE_WITHER_SKELETON`.
pub fn note_block_imitate_wither_skeleton() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.imitate.wither_skeleton"))
}

/// `SoundEvents.NOTE_BLOCK_IMITATE_PIGLIN`.
pub fn note_block_imitate_piglin() -> Holder<SoundEvent> {
    Holder::Key(key("block.note_block.imitate.piglin"))
}

/// `SoundEvents.OCELOT_HURT`.
pub fn ocelot_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ocelot.hurt"))
}

/// `SoundEvents.OCELOT_AMBIENT`.
pub fn ocelot_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ocelot.ambient"))
}

/// `SoundEvents.OCELOT_DEATH`.
pub fn ocelot_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ocelot.death"))
}

/// `SoundEvents.OMINOUS_BOTTLE_DISPOSE`.
pub fn ominous_bottle_dispose() -> Holder<SoundEvent> {
    Holder::Key(key("item.ominous_bottle.dispose"))
}

/// `SoundEvents.PAINTING_BREAK`.
pub fn painting_break() -> Holder<SoundEvent> {
    Holder::Key(key("entity.painting.break"))
}

/// `SoundEvents.PAINTING_PLACE`.
pub fn painting_place() -> Holder<SoundEvent> {
    Holder::Key(key("entity.painting.place"))
}

/// `SoundEvents.PALE_HANGING_MOSS_IDLE`.
pub fn pale_hanging_moss_idle() -> Holder<SoundEvent> {
    Holder::Key(key("block.pale_hanging_moss.idle"))
}

/// `SoundEvents.PANDA_PRE_SNEEZE`.
pub fn panda_pre_sneeze() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.pre_sneeze"))
}

/// `SoundEvents.PANDA_SNEEZE`.
pub fn panda_sneeze() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.sneeze"))
}

/// `SoundEvents.PANDA_AMBIENT`.
pub fn panda_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.ambient"))
}

/// `SoundEvents.PANDA_DEATH`.
pub fn panda_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.death"))
}

/// `SoundEvents.PANDA_EAT`.
pub fn panda_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.eat"))
}

/// `SoundEvents.PANDA_STEP`.
pub fn panda_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.step"))
}

/// `SoundEvents.PANDA_CANT_BREED`.
pub fn panda_cant_breed() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.cant_breed"))
}

/// `SoundEvents.PANDA_AGGRESSIVE_AMBIENT`.
pub fn panda_aggressive_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.aggressive_ambient"))
}

/// `SoundEvents.PANDA_WORRIED_AMBIENT`.
pub fn panda_worried_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.worried_ambient"))
}

/// `SoundEvents.PANDA_HURT`.
pub fn panda_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.hurt"))
}

/// `SoundEvents.PANDA_BITE`.
pub fn panda_bite() -> Holder<SoundEvent> {
    Holder::Key(key("entity.panda.bite"))
}

/// `SoundEvents.PARCHED_AMBIENT`.
pub fn parched_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parched.ambient"))
}

/// `SoundEvents.PARCHED_DEATH`.
pub fn parched_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parched.death"))
}

/// `SoundEvents.PARCHED_HURT`.
pub fn parched_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parched.hurt"))
}

/// `SoundEvents.PARCHED_STEP`.
pub fn parched_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parched.step"))
}

/// `SoundEvents.PARROT_AMBIENT`.
pub fn parrot_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.ambient"))
}

/// `SoundEvents.PARROT_DEATH`.
pub fn parrot_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.death"))
}

/// `SoundEvents.PARROT_EAT`.
pub fn parrot_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.eat"))
}

/// `SoundEvents.PARROT_FLY`.
pub fn parrot_fly() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.fly"))
}

/// `SoundEvents.PARROT_HURT`.
pub fn parrot_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.hurt"))
}

/// `SoundEvents.PARROT_IMITATE_BLAZE`.
pub fn parrot_imitate_blaze() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.blaze"))
}

/// `SoundEvents.PARROT_IMITATE_BOGGED`.
pub fn parrot_imitate_bogged() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.bogged"))
}

/// `SoundEvents.PARROT_IMITATE_BREEZE`.
pub fn parrot_imitate_breeze() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.breeze"))
}

/// `SoundEvents.PARROT_IMITATE_CAMEL_HUSK`.
pub fn parrot_imitate_camel_husk() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.camel_husk"))
}

/// `SoundEvents.PARROT_IMITATE_CREAKING`.
pub fn parrot_imitate_creaking() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.creaking"))
}

/// `SoundEvents.PARROT_IMITATE_CREEPER`.
pub fn parrot_imitate_creeper() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.creeper"))
}

/// `SoundEvents.PARROT_IMITATE_DROWNED`.
pub fn parrot_imitate_drowned() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.drowned"))
}

/// `SoundEvents.PARROT_IMITATE_ELDER_GUARDIAN`.
pub fn parrot_imitate_elder_guardian() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.elder_guardian"))
}

/// `SoundEvents.PARROT_IMITATE_ENDER_DRAGON`.
pub fn parrot_imitate_ender_dragon() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.ender_dragon"))
}

/// `SoundEvents.PARROT_IMITATE_ENDERMITE`.
pub fn parrot_imitate_endermite() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.endermite"))
}

/// `SoundEvents.PARROT_IMITATE_EVOKER`.
pub fn parrot_imitate_evoker() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.evoker"))
}

/// `SoundEvents.PARROT_IMITATE_GHAST`.
pub fn parrot_imitate_ghast() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.ghast"))
}

/// `SoundEvents.PARROT_IMITATE_GUARDIAN`.
pub fn parrot_imitate_guardian() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.guardian"))
}

/// `SoundEvents.PARROT_IMITATE_HOGLIN`.
pub fn parrot_imitate_hoglin() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.hoglin"))
}

/// `SoundEvents.PARROT_IMITATE_HUSK`.
pub fn parrot_imitate_husk() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.husk"))
}

/// `SoundEvents.PARROT_IMITATE_ILLUSIONER`.
pub fn parrot_imitate_illusioner() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.illusioner"))
}

/// `SoundEvents.PARROT_IMITATE_MAGMA_CUBE`.
pub fn parrot_imitate_magma_cube() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.magma_cube"))
}

/// `SoundEvents.PARROT_IMITATE_PHANTOM`.
pub fn parrot_imitate_phantom() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.phantom"))
}

/// `SoundEvents.PARROT_IMITATE_PARCHED`.
pub fn parrot_imitate_parched() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.parched"))
}

/// `SoundEvents.PARROT_IMITATE_PIGLIN`.
pub fn parrot_imitate_piglin() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.piglin"))
}

/// `SoundEvents.PARROT_IMITATE_PIGLIN_BRUTE`.
pub fn parrot_imitate_piglin_brute() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.piglin_brute"))
}

/// `SoundEvents.PARROT_IMITATE_PILLAGER`.
pub fn parrot_imitate_pillager() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.pillager"))
}

/// `SoundEvents.PARROT_IMITATE_RAVAGER`.
pub fn parrot_imitate_ravager() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.ravager"))
}

/// `SoundEvents.PARROT_IMITATE_SHULKER`.
pub fn parrot_imitate_shulker() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.shulker"))
}

/// `SoundEvents.PARROT_IMITATE_SILVERFISH`.
pub fn parrot_imitate_silverfish() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.silverfish"))
}

/// `SoundEvents.PARROT_IMITATE_SKELETON`.
pub fn parrot_imitate_skeleton() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.skeleton"))
}

/// `SoundEvents.PARROT_IMITATE_SLIME`.
pub fn parrot_imitate_slime() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.slime"))
}

/// `SoundEvents.PARROT_IMITATE_SPIDER`.
pub fn parrot_imitate_spider() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.spider"))
}

/// `SoundEvents.PARROT_IMITATE_STRAY`.
pub fn parrot_imitate_stray() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.stray"))
}

/// `SoundEvents.PARROT_IMITATE_VEX`.
pub fn parrot_imitate_vex() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.vex"))
}

/// `SoundEvents.PARROT_IMITATE_VINDICATOR`.
pub fn parrot_imitate_vindicator() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.vindicator"))
}

/// `SoundEvents.PARROT_IMITATE_WARDEN`.
pub fn parrot_imitate_warden() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.warden"))
}

/// `SoundEvents.PARROT_IMITATE_WITCH`.
pub fn parrot_imitate_witch() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.witch"))
}

/// `SoundEvents.PARROT_IMITATE_WITHER`.
pub fn parrot_imitate_wither() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.wither"))
}

/// `SoundEvents.PARROT_IMITATE_WITHER_SKELETON`.
pub fn parrot_imitate_wither_skeleton() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.wither_skeleton"))
}

/// `SoundEvents.PARROT_IMITATE_ZOGLIN`.
pub fn parrot_imitate_zoglin() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.zoglin"))
}

/// `SoundEvents.PARROT_IMITATE_ZOMBIE`.
pub fn parrot_imitate_zombie() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.zombie"))
}

/// `SoundEvents.PARROT_IMITATE_ZOMBIE_HORSE`.
pub fn parrot_imitate_zombie_horse() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.zombie_horse"))
}

/// `SoundEvents.PARROT_IMITATE_ZOMBIE_NAUTILUS`.
pub fn parrot_imitate_zombie_nautilus() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.zombie_nautilus"))
}

/// `SoundEvents.PARROT_IMITATE_ZOMBIE_VILLAGER`.
pub fn parrot_imitate_zombie_villager() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.imitate.zombie_villager"))
}

/// `SoundEvents.PARROT_STEP`.
pub fn parrot_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.parrot.step"))
}

/// `SoundEvents.PHANTOM_AMBIENT`.
pub fn phantom_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.phantom.ambient"))
}

/// `SoundEvents.PHANTOM_BITE`.
pub fn phantom_bite() -> Holder<SoundEvent> {
    Holder::Key(key("entity.phantom.bite"))
}

/// `SoundEvents.PHANTOM_DEATH`.
pub fn phantom_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.phantom.death"))
}

/// `SoundEvents.PHANTOM_FLAP`.
pub fn phantom_flap() -> Holder<SoundEvent> {
    Holder::Key(key("entity.phantom.flap"))
}

/// `SoundEvents.PHANTOM_HURT`.
pub fn phantom_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.phantom.hurt"))
}

/// `SoundEvents.PHANTOM_SWOOP`.
pub fn phantom_swoop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.phantom.swoop"))
}

/// `SoundEvents.PIG_AMBIENT`.
pub fn pig_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pig.ambient"))
}

/// `SoundEvents.PIG_DEATH`.
pub fn pig_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pig.death"))
}

/// `SoundEvents.PIG_HURT`.
pub fn pig_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pig.hurt"))
}

/// `SoundEvents.PIG_SADDLE`.
pub fn pig_saddle() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pig.saddle"))
}

/// `SoundEvents.PIG_STEP`.
pub fn pig_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pig.step"))
}

/// `SoundEvents.PIGLIN_ADMIRING_ITEM`.
pub fn piglin_admiring_item() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.admiring_item"))
}

/// `SoundEvents.PIGLIN_AMBIENT`.
pub fn piglin_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.ambient"))
}

/// `SoundEvents.PIGLIN_ANGRY`.
pub fn piglin_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.angry"))
}

/// `SoundEvents.PIGLIN_CELEBRATE`.
pub fn piglin_celebrate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.celebrate"))
}

/// `SoundEvents.PIGLIN_DEATH`.
pub fn piglin_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.death"))
}

/// `SoundEvents.PIGLIN_JEALOUS`.
pub fn piglin_jealous() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.jealous"))
}

/// `SoundEvents.PIGLIN_HURT`.
pub fn piglin_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.hurt"))
}

/// `SoundEvents.PIGLIN_RETREAT`.
pub fn piglin_retreat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.retreat"))
}

/// `SoundEvents.PIGLIN_STEP`.
pub fn piglin_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.step"))
}

/// `SoundEvents.PIGLIN_CONVERTED_TO_ZOMBIFIED`.
pub fn piglin_converted_to_zombified() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin.converted_to_zombified"))
}

/// `SoundEvents.PIGLIN_BRUTE_AMBIENT`.
pub fn piglin_brute_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin_brute.ambient"))
}

/// `SoundEvents.PIGLIN_BRUTE_ANGRY`.
pub fn piglin_brute_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin_brute.angry"))
}

/// `SoundEvents.PIGLIN_BRUTE_DEATH`.
pub fn piglin_brute_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin_brute.death"))
}

/// `SoundEvents.PIGLIN_BRUTE_HURT`.
pub fn piglin_brute_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin_brute.hurt"))
}

/// `SoundEvents.PIGLIN_BRUTE_STEP`.
pub fn piglin_brute_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin_brute.step"))
}

/// `SoundEvents.PIGLIN_BRUTE_CONVERTED_TO_ZOMBIFIED`.
pub fn piglin_brute_converted_to_zombified() -> Holder<SoundEvent> {
    Holder::Key(key("entity.piglin_brute.converted_to_zombified"))
}

/// `SoundEvents.PILLAGER_AMBIENT`.
pub fn pillager_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pillager.ambient"))
}

/// `SoundEvents.PILLAGER_CELEBRATE`.
pub fn pillager_celebrate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pillager.celebrate"))
}

/// `SoundEvents.PILLAGER_DEATH`.
pub fn pillager_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pillager.death"))
}

/// `SoundEvents.PILLAGER_HURT`.
pub fn pillager_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.pillager.hurt"))
}

/// `SoundEvents.PISTON_CONTRACT`.
pub fn piston_contract() -> Holder<SoundEvent> {
    Holder::Key(key("block.piston.contract"))
}

/// `SoundEvents.PISTON_EXTEND`.
pub fn piston_extend() -> Holder<SoundEvent> {
    Holder::Key(key("block.piston.extend"))
}

/// `SoundEvents.PLAYER_ATTACK_CRIT`.
pub fn player_attack_crit() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.attack.crit"))
}

/// `SoundEvents.PLAYER_ATTACK_KNOCKBACK`.
pub fn player_attack_knockback() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.attack.knockback"))
}

/// `SoundEvents.PLAYER_ATTACK_NODAMAGE`.
pub fn player_attack_nodamage() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.attack.nodamage"))
}

/// `SoundEvents.PLAYER_ATTACK_STRONG`.
pub fn player_attack_strong() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.attack.strong"))
}

/// `SoundEvents.PLAYER_ATTACK_SWEEP`.
pub fn player_attack_sweep() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.attack.sweep"))
}

/// `SoundEvents.PLAYER_ATTACK_WEAK`.
pub fn player_attack_weak() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.attack.weak"))
}

/// `SoundEvents.PLAYER_BIG_FALL`.
pub fn player_big_fall() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.big_fall"))
}

/// `SoundEvents.PLAYER_BREATH`.
pub fn player_breath() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.breath"))
}

/// `SoundEvents.PLAYER_BURP`.
pub fn player_burp() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.burp"))
}

/// `SoundEvents.PLAYER_DEATH`.
pub fn player_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.death"))
}

/// `SoundEvents.PLAYER_HURT`.
pub fn player_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.hurt"))
}

/// `SoundEvents.PLAYER_HURT_DROWN`.
pub fn player_hurt_drown() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.hurt_drown"))
}

/// `SoundEvents.PLAYER_HURT_FREEZE`.
pub fn player_hurt_freeze() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.hurt_freeze"))
}

/// `SoundEvents.PLAYER_HURT_ON_FIRE`.
pub fn player_hurt_on_fire() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.hurt_on_fire"))
}

/// `SoundEvents.PLAYER_HURT_SWEET_BERRY_BUSH`.
pub fn player_hurt_sweet_berry_bush() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.hurt_sweet_berry_bush"))
}

/// `SoundEvents.PLAYER_LEVELUP`.
pub fn player_levelup() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.levelup"))
}

/// `SoundEvents.PLAYER_SMALL_FALL`.
pub fn player_small_fall() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.small_fall"))
}

/// `SoundEvents.PLAYER_SPLASH`.
pub fn player_splash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.splash"))
}

/// `SoundEvents.PLAYER_SPLASH_HIGH_SPEED`.
pub fn player_splash_high_speed() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.splash.high_speed"))
}

/// `SoundEvents.PLAYER_SWIM`.
pub fn player_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.swim"))
}

/// `SoundEvents.PLAYER_TELEPORT`.
pub fn player_teleport() -> Holder<SoundEvent> {
    Holder::Key(key("entity.player.teleport"))
}

/// `SoundEvents.POLAR_BEAR_AMBIENT`.
pub fn polar_bear_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.polar_bear.ambient"))
}

/// `SoundEvents.POLAR_BEAR_AMBIENT_BABY`.
pub fn polar_bear_ambient_baby() -> Holder<SoundEvent> {
    Holder::Key(key("entity.polar_bear.ambient_baby"))
}

/// `SoundEvents.POLAR_BEAR_DEATH`.
pub fn polar_bear_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.polar_bear.death"))
}

/// `SoundEvents.POLAR_BEAR_HURT`.
pub fn polar_bear_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.polar_bear.hurt"))
}

/// `SoundEvents.POLAR_BEAR_STEP`.
pub fn polar_bear_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.polar_bear.step"))
}

/// `SoundEvents.POLAR_BEAR_WARNING`.
pub fn polar_bear_warning() -> Holder<SoundEvent> {
    Holder::Key(key("entity.polar_bear.warning"))
}

/// `SoundEvents.POLISHED_DEEPSLATE_BREAK`.
pub fn polished_deepslate_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_deepslate.break"))
}

/// `SoundEvents.POLISHED_DEEPSLATE_FALL`.
pub fn polished_deepslate_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_deepslate.fall"))
}

/// `SoundEvents.POLISHED_DEEPSLATE_HIT`.
pub fn polished_deepslate_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_deepslate.hit"))
}

/// `SoundEvents.POLISHED_DEEPSLATE_PLACE`.
pub fn polished_deepslate_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_deepslate.place"))
}

/// `SoundEvents.POLISHED_DEEPSLATE_STEP`.
pub fn polished_deepslate_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_deepslate.step"))
}

/// `SoundEvents.PORTAL_AMBIENT`.
pub fn portal_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.portal.ambient"))
}

/// `SoundEvents.PORTAL_TRAVEL`.
pub fn portal_travel() -> Holder<SoundEvent> {
    Holder::Key(key("block.portal.travel"))
}

/// `SoundEvents.PORTAL_TRIGGER`.
pub fn portal_trigger() -> Holder<SoundEvent> {
    Holder::Key(key("block.portal.trigger"))
}

/// `SoundEvents.POWDER_SNOW_BREAK`.
pub fn powder_snow_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.powder_snow.break"))
}

/// `SoundEvents.POWDER_SNOW_FALL`.
pub fn powder_snow_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.powder_snow.fall"))
}

/// `SoundEvents.POWDER_SNOW_HIT`.
pub fn powder_snow_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.powder_snow.hit"))
}

/// `SoundEvents.POWDER_SNOW_PLACE`.
pub fn powder_snow_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.powder_snow.place"))
}

/// `SoundEvents.POWDER_SNOW_STEP`.
pub fn powder_snow_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.powder_snow.step"))
}

/// `SoundEvents.PUFFER_FISH_BLOW_OUT`.
pub fn puffer_fish_blow_out() -> Holder<SoundEvent> {
    Holder::Key(key("entity.puffer_fish.blow_out"))
}

/// `SoundEvents.PUFFER_FISH_BLOW_UP`.
pub fn puffer_fish_blow_up() -> Holder<SoundEvent> {
    Holder::Key(key("entity.puffer_fish.blow_up"))
}

/// `SoundEvents.PUFFER_FISH_DEATH`.
pub fn puffer_fish_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.puffer_fish.death"))
}

/// `SoundEvents.PUFFER_FISH_FLOP`.
pub fn puffer_fish_flop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.puffer_fish.flop"))
}

/// `SoundEvents.PUFFER_FISH_HURT`.
pub fn puffer_fish_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.puffer_fish.hurt"))
}

/// `SoundEvents.PUFFER_FISH_STING`.
pub fn puffer_fish_sting() -> Holder<SoundEvent> {
    Holder::Key(key("entity.puffer_fish.sting"))
}

/// `SoundEvents.PUMPKIN_CARVE`.
pub fn pumpkin_carve() -> Holder<SoundEvent> {
    Holder::Key(key("block.pumpkin.carve"))
}

/// `SoundEvents.RABBIT_AMBIENT`.
pub fn rabbit_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.rabbit.ambient"))
}

/// `SoundEvents.RABBIT_ATTACK`.
pub fn rabbit_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.rabbit.attack"))
}

/// `SoundEvents.RABBIT_DEATH`.
pub fn rabbit_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.rabbit.death"))
}

/// `SoundEvents.RABBIT_HURT`.
pub fn rabbit_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.rabbit.hurt"))
}

/// `SoundEvents.RABBIT_JUMP`.
pub fn rabbit_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.rabbit.jump"))
}

/// `SoundEvents.RAID_HORN`.
pub fn raid_horn() -> Holder<SoundEvent> {
    Holder::Key(key("event.raid.horn"))
}

/// `SoundEvents.RAVAGER_AMBIENT`.
pub fn ravager_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ravager.ambient"))
}

/// `SoundEvents.RAVAGER_ATTACK`.
pub fn ravager_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ravager.attack"))
}

/// `SoundEvents.RAVAGER_CELEBRATE`.
pub fn ravager_celebrate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ravager.celebrate"))
}

/// `SoundEvents.RAVAGER_DEATH`.
pub fn ravager_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ravager.death"))
}

/// `SoundEvents.RAVAGER_HURT`.
pub fn ravager_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ravager.hurt"))
}

/// `SoundEvents.RAVAGER_STEP`.
pub fn ravager_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ravager.step"))
}

/// `SoundEvents.RAVAGER_STUNNED`.
pub fn ravager_stunned() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ravager.stunned"))
}

/// `SoundEvents.RAVAGER_ROAR`.
pub fn ravager_roar() -> Holder<SoundEvent> {
    Holder::Key(key("entity.ravager.roar"))
}

/// `SoundEvents.NETHER_GOLD_ORE_BREAK`.
pub fn nether_gold_ore_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_gold_ore.break"))
}

/// `SoundEvents.NETHER_GOLD_ORE_FALL`.
pub fn nether_gold_ore_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_gold_ore.fall"))
}

/// `SoundEvents.NETHER_GOLD_ORE_HIT`.
pub fn nether_gold_ore_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_gold_ore.hit"))
}

/// `SoundEvents.NETHER_GOLD_ORE_PLACE`.
pub fn nether_gold_ore_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_gold_ore.place"))
}

/// `SoundEvents.NETHER_GOLD_ORE_STEP`.
pub fn nether_gold_ore_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_gold_ore.step"))
}

/// `SoundEvents.NETHER_ORE_BREAK`.
pub fn nether_ore_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_ore.break"))
}

/// `SoundEvents.NETHER_ORE_FALL`.
pub fn nether_ore_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_ore.fall"))
}

/// `SoundEvents.NETHER_ORE_HIT`.
pub fn nether_ore_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_ore.hit"))
}

/// `SoundEvents.NETHER_ORE_PLACE`.
pub fn nether_ore_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_ore.place"))
}

/// `SoundEvents.NETHER_ORE_STEP`.
pub fn nether_ore_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.nether_ore.step"))
}

/// `SoundEvents.REDSTONE_TORCH_BURNOUT`.
pub fn redstone_torch_burnout() -> Holder<SoundEvent> {
    Holder::Key(key("block.redstone_torch.burnout"))
}

/// `SoundEvents.RESIN_BREAK`.
pub fn resin_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin.break"))
}

/// `SoundEvents.RESIN_FALL`.
pub fn resin_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin.fall"))
}

/// `SoundEvents.RESIN_PLACE`.
pub fn resin_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin.place"))
}

/// `SoundEvents.RESIN_STEP`.
pub fn resin_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin.step"))
}

/// `SoundEvents.RESIN_BRICKS_BREAK`.
pub fn resin_bricks_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin_bricks.break"))
}

/// `SoundEvents.RESIN_BRICKS_FALL`.
pub fn resin_bricks_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin_bricks.fall"))
}

/// `SoundEvents.RESIN_BRICKS_HIT`.
pub fn resin_bricks_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin_bricks.hit"))
}

/// `SoundEvents.RESIN_BRICKS_PLACE`.
pub fn resin_bricks_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin_bricks.place"))
}

/// `SoundEvents.RESIN_BRICKS_STEP`.
pub fn resin_bricks_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.resin_bricks.step"))
}

/// `SoundEvents.RESPAWN_ANCHOR_AMBIENT`.
pub fn respawn_anchor_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.respawn_anchor.ambient"))
}

/// `SoundEvents.RESPAWN_ANCHOR_CHARGE`.
pub fn respawn_anchor_charge() -> Holder<SoundEvent> {
    Holder::Key(key("block.respawn_anchor.charge"))
}

/// `SoundEvents.RESPAWN_ANCHOR_DEPLETE`.
pub fn respawn_anchor_deplete() -> Holder<SoundEvent> {
    Holder::Key(key("block.respawn_anchor.deplete"))
}

/// `SoundEvents.RESPAWN_ANCHOR_SET_SPAWN`.
pub fn respawn_anchor_set_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("block.respawn_anchor.set_spawn"))
}

/// `SoundEvents.ROOTED_DIRT_BREAK`.
pub fn rooted_dirt_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.rooted_dirt.break"))
}

/// `SoundEvents.ROOTED_DIRT_FALL`.
pub fn rooted_dirt_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.rooted_dirt.fall"))
}

/// `SoundEvents.ROOTED_DIRT_HIT`.
pub fn rooted_dirt_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.rooted_dirt.hit"))
}

/// `SoundEvents.ROOTED_DIRT_PLACE`.
pub fn rooted_dirt_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.rooted_dirt.place"))
}

/// `SoundEvents.ROOTED_DIRT_STEP`.
pub fn rooted_dirt_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.rooted_dirt.step"))
}

/// `SoundEvents.SALMON_AMBIENT`.
pub fn salmon_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.salmon.ambient"))
}

/// `SoundEvents.SALMON_DEATH`.
pub fn salmon_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.salmon.death"))
}

/// `SoundEvents.SALMON_FLOP`.
pub fn salmon_flop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.salmon.flop"))
}

/// `SoundEvents.SALMON_HURT`.
pub fn salmon_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.salmon.hurt"))
}

/// `SoundEvents.SAND_BREAK`.
pub fn sand_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.sand.break"))
}

/// `SoundEvents.SAND_FALL`.
pub fn sand_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.sand.fall"))
}

/// `SoundEvents.SAND_HIT`.
pub fn sand_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.sand.hit"))
}

/// `SoundEvents.SAND_PLACE`.
pub fn sand_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.sand.place"))
}

/// `SoundEvents.SAND_STEP`.
pub fn sand_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.sand.step"))
}

/// `SoundEvents.SAND_IDLE`.
pub fn sand_idle() -> Holder<SoundEvent> {
    Holder::Key(key("block.sand.idle"))
}

/// `SoundEvents.SCAFFOLDING_BREAK`.
pub fn scaffolding_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.scaffolding.break"))
}

/// `SoundEvents.SCAFFOLDING_FALL`.
pub fn scaffolding_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.scaffolding.fall"))
}

/// `SoundEvents.SCAFFOLDING_HIT`.
pub fn scaffolding_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.scaffolding.hit"))
}

/// `SoundEvents.SCAFFOLDING_PLACE`.
pub fn scaffolding_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.scaffolding.place"))
}

/// `SoundEvents.SCAFFOLDING_STEP`.
pub fn scaffolding_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.scaffolding.step"))
}

/// `SoundEvents.SCULK_BLOCK_SPREAD`.
pub fn sculk_block_spread() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk.spread"))
}

/// `SoundEvents.SCULK_BLOCK_CHARGE`.
pub fn sculk_block_charge() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk.charge"))
}

/// `SoundEvents.SCULK_BLOCK_BREAK`.
pub fn sculk_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk.break"))
}

/// `SoundEvents.SCULK_BLOCK_FALL`.
pub fn sculk_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk.fall"))
}

/// `SoundEvents.SCULK_BLOCK_HIT`.
pub fn sculk_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk.hit"))
}

/// `SoundEvents.SCULK_BLOCK_PLACE`.
pub fn sculk_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk.place"))
}

/// `SoundEvents.SCULK_BLOCK_STEP`.
pub fn sculk_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk.step"))
}

/// `SoundEvents.SCULK_CATALYST_BLOOM`.
pub fn sculk_catalyst_bloom() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_catalyst.bloom"))
}

/// `SoundEvents.SCULK_CATALYST_BREAK`.
pub fn sculk_catalyst_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_catalyst.break"))
}

/// `SoundEvents.SCULK_CATALYST_FALL`.
pub fn sculk_catalyst_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_catalyst.fall"))
}

/// `SoundEvents.SCULK_CATALYST_HIT`.
pub fn sculk_catalyst_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_catalyst.hit"))
}

/// `SoundEvents.SCULK_CATALYST_PLACE`.
pub fn sculk_catalyst_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_catalyst.place"))
}

/// `SoundEvents.SCULK_CATALYST_STEP`.
pub fn sculk_catalyst_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_catalyst.step"))
}

/// `SoundEvents.SCULK_CLICKING`.
pub fn sculk_clicking() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_sensor.clicking"))
}

/// `SoundEvents.SCULK_CLICKING_STOP`.
pub fn sculk_clicking_stop() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_sensor.clicking_stop"))
}

/// `SoundEvents.SCULK_SENSOR_BREAK`.
pub fn sculk_sensor_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_sensor.break"))
}

/// `SoundEvents.SCULK_SENSOR_FALL`.
pub fn sculk_sensor_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_sensor.fall"))
}

/// `SoundEvents.SCULK_SENSOR_HIT`.
pub fn sculk_sensor_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_sensor.hit"))
}

/// `SoundEvents.SCULK_SENSOR_PLACE`.
pub fn sculk_sensor_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_sensor.place"))
}

/// `SoundEvents.SCULK_SENSOR_STEP`.
pub fn sculk_sensor_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_sensor.step"))
}

/// `SoundEvents.SCULK_SHRIEKER_BREAK`.
pub fn sculk_shrieker_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_shrieker.break"))
}

/// `SoundEvents.SCULK_SHRIEKER_FALL`.
pub fn sculk_shrieker_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_shrieker.fall"))
}

/// `SoundEvents.SCULK_SHRIEKER_HIT`.
pub fn sculk_shrieker_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_shrieker.hit"))
}

/// `SoundEvents.SCULK_SHRIEKER_PLACE`.
pub fn sculk_shrieker_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_shrieker.place"))
}

/// `SoundEvents.SCULK_SHRIEKER_SHRIEK`.
pub fn sculk_shrieker_shriek() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_shrieker.shriek"))
}

/// `SoundEvents.SCULK_SHRIEKER_STEP`.
pub fn sculk_shrieker_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_shrieker.step"))
}

/// `SoundEvents.SCULK_VEIN_BREAK`.
pub fn sculk_vein_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_vein.break"))
}

/// `SoundEvents.SCULK_VEIN_FALL`.
pub fn sculk_vein_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_vein.fall"))
}

/// `SoundEvents.SCULK_VEIN_HIT`.
pub fn sculk_vein_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_vein.hit"))
}

/// `SoundEvents.SCULK_VEIN_PLACE`.
pub fn sculk_vein_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_vein.place"))
}

/// `SoundEvents.SCULK_VEIN_STEP`.
pub fn sculk_vein_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.sculk_vein.step"))
}

/// `SoundEvents.SHEEP_AMBIENT`.
pub fn sheep_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sheep.ambient"))
}

/// `SoundEvents.SHEEP_DEATH`.
pub fn sheep_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sheep.death"))
}

/// `SoundEvents.SHEEP_HURT`.
pub fn sheep_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sheep.hurt"))
}

/// `SoundEvents.SHEEP_SHEAR`.
pub fn sheep_shear() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sheep.shear"))
}

/// `SoundEvents.SHEEP_STEP`.
pub fn sheep_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sheep.step"))
}

/// `SoundEvents.SHEARS_SNIP`.
pub fn shears_snip() -> Holder<SoundEvent> {
    Holder::Key(key("item.shears.snip"))
}

/// `SoundEvents.SHELF_ACTIVATE`.
pub fn shelf_activate() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.activate"))
}

/// `SoundEvents.SHELF_BREAK`.
pub fn shelf_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.break"))
}

/// `SoundEvents.SHELF_DEACTIVATE`.
pub fn shelf_deactivate() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.deactivate"))
}

/// `SoundEvents.SHELF_FALL`.
pub fn shelf_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.fall"))
}

/// `SoundEvents.SHELF_HIT`.
pub fn shelf_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.hit"))
}

/// `SoundEvents.SHELF_MULTI_SWAP`.
pub fn shelf_multi_swap() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.multi_swap"))
}

/// `SoundEvents.SHELF_PLACE`.
pub fn shelf_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.place"))
}

/// `SoundEvents.SHELF_PLACE_ITEM`.
pub fn shelf_place_item() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.place_item"))
}

/// `SoundEvents.SHELF_SINGLE_SWAP`.
pub fn shelf_single_swap() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.single_swap"))
}

/// `SoundEvents.SHELF_STEP`.
pub fn shelf_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.step"))
}

/// `SoundEvents.SHELF_TAKE_ITEM`.
pub fn shelf_take_item() -> Holder<SoundEvent> {
    Holder::Key(key("block.shelf.take_item"))
}

/// `SoundEvents.SHIELD_BLOCK`.
pub fn shield_block() -> Holder<SoundEvent> {
    Holder::Key(key("item.shield.block"))
}

/// `SoundEvents.SHIELD_BREAK`.
pub fn shield_break() -> Holder<SoundEvent> {
    Holder::Key(key("item.shield.break"))
}

/// `SoundEvents.SHROOMLIGHT_BREAK`.
pub fn shroomlight_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.shroomlight.break"))
}

/// `SoundEvents.SHROOMLIGHT_STEP`.
pub fn shroomlight_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.shroomlight.step"))
}

/// `SoundEvents.SHROOMLIGHT_PLACE`.
pub fn shroomlight_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.shroomlight.place"))
}

/// `SoundEvents.SHROOMLIGHT_HIT`.
pub fn shroomlight_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.shroomlight.hit"))
}

/// `SoundEvents.SHROOMLIGHT_FALL`.
pub fn shroomlight_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.shroomlight.fall"))
}

/// `SoundEvents.SHOVEL_FLATTEN`.
pub fn shovel_flatten() -> Holder<SoundEvent> {
    Holder::Key(key("item.shovel.flatten"))
}

/// `SoundEvents.SHULKER_AMBIENT`.
pub fn shulker_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker.ambient"))
}

/// `SoundEvents.SHULKER_BOX_CLOSE`.
pub fn shulker_box_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.shulker_box.close"))
}

/// `SoundEvents.SHULKER_BOX_OPEN`.
pub fn shulker_box_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.shulker_box.open"))
}

/// `SoundEvents.SHULKER_BULLET_HIT`.
pub fn shulker_bullet_hit() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker_bullet.hit"))
}

/// `SoundEvents.SHULKER_BULLET_HURT`.
pub fn shulker_bullet_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker_bullet.hurt"))
}

/// `SoundEvents.SHULKER_CLOSE`.
pub fn shulker_close() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker.close"))
}

/// `SoundEvents.SHULKER_DEATH`.
pub fn shulker_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker.death"))
}

/// `SoundEvents.SHULKER_HURT`.
pub fn shulker_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker.hurt"))
}

/// `SoundEvents.SHULKER_HURT_CLOSED`.
pub fn shulker_hurt_closed() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker.hurt_closed"))
}

/// `SoundEvents.SHULKER_OPEN`.
pub fn shulker_open() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker.open"))
}

/// `SoundEvents.SHULKER_SHOOT`.
pub fn shulker_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker.shoot"))
}

/// `SoundEvents.SHULKER_TELEPORT`.
pub fn shulker_teleport() -> Holder<SoundEvent> {
    Holder::Key(key("entity.shulker.teleport"))
}

/// `SoundEvents.SILVERFISH_AMBIENT`.
pub fn silverfish_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.silverfish.ambient"))
}

/// `SoundEvents.SILVERFISH_DEATH`.
pub fn silverfish_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.silverfish.death"))
}

/// `SoundEvents.SILVERFISH_HURT`.
pub fn silverfish_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.silverfish.hurt"))
}

/// `SoundEvents.SILVERFISH_STEP`.
pub fn silverfish_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.silverfish.step"))
}

/// `SoundEvents.SKELETON_AMBIENT`.
pub fn skeleton_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton.ambient"))
}

/// `SoundEvents.SKELETON_CONVERTED_TO_STRAY`.
pub fn skeleton_converted_to_stray() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton.converted_to_stray"))
}

/// `SoundEvents.SKELETON_DEATH`.
pub fn skeleton_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton.death"))
}

/// `SoundEvents.SKELETON_HORSE_AMBIENT`.
pub fn skeleton_horse_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton_horse.ambient"))
}

/// `SoundEvents.SKELETON_HORSE_DEATH`.
pub fn skeleton_horse_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton_horse.death"))
}

/// `SoundEvents.SKELETON_HORSE_HURT`.
pub fn skeleton_horse_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton_horse.hurt"))
}

/// `SoundEvents.SKELETON_HORSE_SWIM`.
pub fn skeleton_horse_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton_horse.swim"))
}

/// `SoundEvents.SKELETON_HORSE_AMBIENT_WATER`.
pub fn skeleton_horse_ambient_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton_horse.ambient_water"))
}

/// `SoundEvents.SKELETON_HORSE_GALLOP_WATER`.
pub fn skeleton_horse_gallop_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton_horse.gallop_water"))
}

/// `SoundEvents.SKELETON_HORSE_JUMP_WATER`.
pub fn skeleton_horse_jump_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton_horse.jump_water"))
}

/// `SoundEvents.SKELETON_HORSE_STEP_WATER`.
pub fn skeleton_horse_step_water() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton_horse.step_water"))
}

/// `SoundEvents.SKELETON_HURT`.
pub fn skeleton_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton.hurt"))
}

/// `SoundEvents.SKELETON_SHOOT`.
pub fn skeleton_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton.shoot"))
}

/// `SoundEvents.SKELETON_STEP`.
pub fn skeleton_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.skeleton.step"))
}

/// `SoundEvents.SLIME_ATTACK`.
pub fn slime_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.attack"))
}

/// `SoundEvents.SLIME_DEATH`.
pub fn slime_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.death"))
}

/// `SoundEvents.SLIME_HURT`.
pub fn slime_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.hurt"))
}

/// `SoundEvents.SLIME_JUMP`.
pub fn slime_jump() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.jump"))
}

/// `SoundEvents.SLIME_SQUISH`.
pub fn slime_squish() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.squish"))
}

/// `SoundEvents.SLIME_BLOCK_BREAK`.
pub fn slime_block_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.slime_block.break"))
}

/// `SoundEvents.SLIME_BLOCK_FALL`.
pub fn slime_block_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.slime_block.fall"))
}

/// `SoundEvents.SLIME_BLOCK_HIT`.
pub fn slime_block_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.slime_block.hit"))
}

/// `SoundEvents.SLIME_BLOCK_PLACE`.
pub fn slime_block_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.slime_block.place"))
}

/// `SoundEvents.SLIME_BLOCK_STEP`.
pub fn slime_block_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.slime_block.step"))
}

/// `SoundEvents.SMALL_AMETHYST_BUD_BREAK`.
pub fn small_amethyst_bud_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.small_amethyst_bud.break"))
}

/// `SoundEvents.SMALL_AMETHYST_BUD_PLACE`.
pub fn small_amethyst_bud_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.small_amethyst_bud.place"))
}

/// `SoundEvents.SMALL_DRIPLEAF_BREAK`.
pub fn small_dripleaf_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.small_dripleaf.break"))
}

/// `SoundEvents.SMALL_DRIPLEAF_FALL`.
pub fn small_dripleaf_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.small_dripleaf.fall"))
}

/// `SoundEvents.SMALL_DRIPLEAF_HIT`.
pub fn small_dripleaf_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.small_dripleaf.hit"))
}

/// `SoundEvents.SMALL_DRIPLEAF_PLACE`.
pub fn small_dripleaf_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.small_dripleaf.place"))
}

/// `SoundEvents.SMALL_DRIPLEAF_STEP`.
pub fn small_dripleaf_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.small_dripleaf.step"))
}

/// `SoundEvents.SOUL_SAND_BREAK`.
pub fn soul_sand_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_sand.break"))
}

/// `SoundEvents.SOUL_SAND_STEP`.
pub fn soul_sand_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_sand.step"))
}

/// `SoundEvents.SOUL_SAND_PLACE`.
pub fn soul_sand_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_sand.place"))
}

/// `SoundEvents.SOUL_SAND_HIT`.
pub fn soul_sand_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_sand.hit"))
}

/// `SoundEvents.SOUL_SAND_FALL`.
pub fn soul_sand_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_sand.fall"))
}

/// `SoundEvents.SOUL_SOIL_BREAK`.
pub fn soul_soil_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_soil.break"))
}

/// `SoundEvents.SOUL_SOIL_STEP`.
pub fn soul_soil_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_soil.step"))
}

/// `SoundEvents.SOUL_SOIL_PLACE`.
pub fn soul_soil_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_soil.place"))
}

/// `SoundEvents.SOUL_SOIL_HIT`.
pub fn soul_soil_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_soil.hit"))
}

/// `SoundEvents.SOUL_SOIL_FALL`.
pub fn soul_soil_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.soul_soil.fall"))
}

/// `SoundEvents.SOUL_ESCAPE`.
pub fn soul_escape() -> Holder<SoundEvent> {
    Holder::Key(key("particle.soul_escape"))
}

/// `SoundEvents.SPAWNER_BREAK`.
pub fn spawner_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.spawner.break"))
}

/// `SoundEvents.SPAWNER_FALL`.
pub fn spawner_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.spawner.fall"))
}

/// `SoundEvents.SPAWNER_HIT`.
pub fn spawner_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.spawner.hit"))
}

/// `SoundEvents.SPAWNER_PLACE`.
pub fn spawner_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.spawner.place"))
}

/// `SoundEvents.SPAWNER_STEP`.
pub fn spawner_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.spawner.step"))
}

/// `SoundEvents.SPORE_BLOSSOM_BREAK`.
pub fn spore_blossom_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.spore_blossom.break"))
}

/// `SoundEvents.SPORE_BLOSSOM_FALL`.
pub fn spore_blossom_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.spore_blossom.fall"))
}

/// `SoundEvents.SPORE_BLOSSOM_HIT`.
pub fn spore_blossom_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.spore_blossom.hit"))
}

/// `SoundEvents.SPORE_BLOSSOM_PLACE`.
pub fn spore_blossom_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.spore_blossom.place"))
}

/// `SoundEvents.SPORE_BLOSSOM_STEP`.
pub fn spore_blossom_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.spore_blossom.step"))
}

/// `SoundEvents.STRIDER_AMBIENT`.
pub fn strider_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.ambient"))
}

/// `SoundEvents.STRIDER_HAPPY`.
pub fn strider_happy() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.happy"))
}

/// `SoundEvents.STRIDER_RETREAT`.
pub fn strider_retreat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.retreat"))
}

/// `SoundEvents.STRIDER_DEATH`.
pub fn strider_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.death"))
}

/// `SoundEvents.STRIDER_HURT`.
pub fn strider_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.hurt"))
}

/// `SoundEvents.STRIDER_STEP`.
pub fn strider_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.step"))
}

/// `SoundEvents.STRIDER_STEP_LAVA`.
pub fn strider_step_lava() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.step_lava"))
}

/// `SoundEvents.STRIDER_EAT`.
pub fn strider_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.eat"))
}

/// `SoundEvents.STRIDER_SADDLE`.
pub fn strider_saddle() -> Holder<SoundEvent> {
    Holder::Key(key("entity.strider.saddle"))
}

/// `SoundEvents.SLIME_DEATH_SMALL`.
pub fn slime_death_small() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.death_small"))
}

/// `SoundEvents.SLIME_HURT_SMALL`.
pub fn slime_hurt_small() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.hurt_small"))
}

/// `SoundEvents.SLIME_JUMP_SMALL`.
pub fn slime_jump_small() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.jump_small"))
}

/// `SoundEvents.SLIME_SQUISH_SMALL`.
pub fn slime_squish_small() -> Holder<SoundEvent> {
    Holder::Key(key("entity.slime.squish_small"))
}

/// `SoundEvents.SMITHING_TABLE_USE`.
pub fn smithing_table_use() -> Holder<SoundEvent> {
    Holder::Key(key("block.smithing_table.use"))
}

/// `SoundEvents.SMOKER_SMOKE`.
pub fn smoker_smoke() -> Holder<SoundEvent> {
    Holder::Key(key("block.smoker.smoke"))
}

/// `SoundEvents.SNIFFER_STEP`.
pub fn sniffer_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.step"))
}

/// `SoundEvents.SNIFFER_EAT`.
pub fn sniffer_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.eat"))
}

/// `SoundEvents.SNIFFER_IDLE`.
pub fn sniffer_idle() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.idle"))
}

/// `SoundEvents.SNIFFER_HURT`.
pub fn sniffer_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.hurt"))
}

/// `SoundEvents.SNIFFER_DEATH`.
pub fn sniffer_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.death"))
}

/// `SoundEvents.SNIFFER_DROP_SEED`.
pub fn sniffer_drop_seed() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.drop_seed"))
}

/// `SoundEvents.SNIFFER_SCENTING`.
pub fn sniffer_scenting() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.scenting"))
}

/// `SoundEvents.SNIFFER_SNIFFING`.
pub fn sniffer_sniffing() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.sniffing"))
}

/// `SoundEvents.SNIFFER_SEARCHING`.
pub fn sniffer_searching() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.searching"))
}

/// `SoundEvents.SNIFFER_DIGGING`.
pub fn sniffer_digging() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.digging"))
}

/// `SoundEvents.SNIFFER_DIGGING_STOP`.
pub fn sniffer_digging_stop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.digging_stop"))
}

/// `SoundEvents.SNIFFER_HAPPY`.
pub fn sniffer_happy() -> Holder<SoundEvent> {
    Holder::Key(key("entity.sniffer.happy"))
}

/// `SoundEvents.SNIFFER_EGG_PLOP`.
pub fn sniffer_egg_plop() -> Holder<SoundEvent> {
    Holder::Key(key("block.sniffer_egg.plop"))
}

/// `SoundEvents.SNIFFER_EGG_CRACK`.
pub fn sniffer_egg_crack() -> Holder<SoundEvent> {
    Holder::Key(key("block.sniffer_egg.crack"))
}

/// `SoundEvents.SNIFFER_EGG_HATCH`.
pub fn sniffer_egg_hatch() -> Holder<SoundEvent> {
    Holder::Key(key("block.sniffer_egg.hatch"))
}

/// `SoundEvents.SNOWBALL_THROW`.
pub fn snowball_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.snowball.throw"))
}

/// `SoundEvents.SNOW_BREAK`.
pub fn snow_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.snow.break"))
}

/// `SoundEvents.SNOW_FALL`.
pub fn snow_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.snow.fall"))
}

/// `SoundEvents.SNOW_GOLEM_AMBIENT`.
pub fn snow_golem_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.snow_golem.ambient"))
}

/// `SoundEvents.SNOW_GOLEM_DEATH`.
pub fn snow_golem_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.snow_golem.death"))
}

/// `SoundEvents.SNOW_GOLEM_HURT`.
pub fn snow_golem_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.snow_golem.hurt"))
}

/// `SoundEvents.SNOW_GOLEM_SHOOT`.
pub fn snow_golem_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.snow_golem.shoot"))
}

/// `SoundEvents.SNOW_GOLEM_SHEAR`.
pub fn snow_golem_shear() -> Holder<SoundEvent> {
    Holder::Key(key("entity.snow_golem.shear"))
}

/// `SoundEvents.SNOW_HIT`.
pub fn snow_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.snow.hit"))
}

/// `SoundEvents.SNOW_PLACE`.
pub fn snow_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.snow.place"))
}

/// `SoundEvents.SNOW_STEP`.
pub fn snow_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.snow.step"))
}

/// `SoundEvents.SPIDER_AMBIENT`.
pub fn spider_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.spider.ambient"))
}

/// `SoundEvents.SPIDER_DEATH`.
pub fn spider_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.spider.death"))
}

/// `SoundEvents.SPIDER_HURT`.
pub fn spider_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.spider.hurt"))
}

/// `SoundEvents.SPIDER_STEP`.
pub fn spider_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.spider.step"))
}

/// `SoundEvents.SPLASH_POTION_BREAK`.
pub fn splash_potion_break() -> Holder<SoundEvent> {
    Holder::Key(key("entity.splash_potion.break"))
}

/// `SoundEvents.SPLASH_POTION_THROW`.
pub fn splash_potion_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.splash_potion.throw"))
}

/// `SoundEvents.SPONGE_BREAK`.
pub fn sponge_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.sponge.break"))
}

/// `SoundEvents.SPONGE_FALL`.
pub fn sponge_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.sponge.fall"))
}

/// `SoundEvents.SPONGE_HIT`.
pub fn sponge_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.sponge.hit"))
}

/// `SoundEvents.SPONGE_PLACE`.
pub fn sponge_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.sponge.place"))
}

/// `SoundEvents.SPONGE_STEP`.
pub fn sponge_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.sponge.step"))
}

/// `SoundEvents.SPONGE_ABSORB`.
pub fn sponge_absorb() -> Holder<SoundEvent> {
    Holder::Key(key("block.sponge.absorb"))
}

/// `SoundEvents.SPYGLASS_USE`.
pub fn spyglass_use() -> Holder<SoundEvent> {
    Holder::Key(key("item.spyglass.use"))
}

/// `SoundEvents.SPYGLASS_STOP_USING`.
pub fn spyglass_stop_using() -> Holder<SoundEvent> {
    Holder::Key(key("item.spyglass.stop_using"))
}

/// `SoundEvents.SQUID_AMBIENT`.
pub fn squid_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.squid.ambient"))
}

/// `SoundEvents.SQUID_DEATH`.
pub fn squid_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.squid.death"))
}

/// `SoundEvents.SQUID_HURT`.
pub fn squid_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.squid.hurt"))
}

/// `SoundEvents.SQUID_SQUIRT`.
pub fn squid_squirt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.squid.squirt"))
}

/// `SoundEvents.STONE_BREAK`.
pub fn stone_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone.break"))
}

/// `SoundEvents.STONE_BUTTON_CLICK_OFF`.
pub fn stone_button_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone_button.click_off"))
}

/// `SoundEvents.STONE_BUTTON_CLICK_ON`.
pub fn stone_button_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone_button.click_on"))
}

/// `SoundEvents.STONE_FALL`.
pub fn stone_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone.fall"))
}

/// `SoundEvents.STONE_HIT`.
pub fn stone_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone.hit"))
}

/// `SoundEvents.STONE_PLACE`.
pub fn stone_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone.place"))
}

/// `SoundEvents.STONE_PRESSURE_PLATE_CLICK_OFF`.
pub fn stone_pressure_plate_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone_pressure_plate.click_off"))
}

/// `SoundEvents.STONE_PRESSURE_PLATE_CLICK_ON`.
pub fn stone_pressure_plate_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone_pressure_plate.click_on"))
}

/// `SoundEvents.STONE_STEP`.
pub fn stone_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.stone.step"))
}

/// `SoundEvents.STRAY_AMBIENT`.
pub fn stray_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.stray.ambient"))
}

/// `SoundEvents.STRAY_DEATH`.
pub fn stray_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.stray.death"))
}

/// `SoundEvents.STRAY_HURT`.
pub fn stray_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.stray.hurt"))
}

/// `SoundEvents.STRAY_STEP`.
pub fn stray_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.stray.step"))
}

/// `SoundEvents.SWEET_BERRY_BUSH_BREAK`.
pub fn sweet_berry_bush_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.sweet_berry_bush.break"))
}

/// `SoundEvents.SWEET_BERRY_BUSH_PLACE`.
pub fn sweet_berry_bush_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.sweet_berry_bush.place"))
}

/// `SoundEvents.SWEET_BERRY_BUSH_PICK_BERRIES`.
pub fn sweet_berry_bush_pick_berries() -> Holder<SoundEvent> {
    Holder::Key(key("block.sweet_berry_bush.pick_berries"))
}

/// `SoundEvents.TADPOLE_DEATH`.
pub fn tadpole_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tadpole.death"))
}

/// `SoundEvents.TADPOLE_FLOP`.
pub fn tadpole_flop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tadpole.flop"))
}

/// `SoundEvents.TADPOLE_GROW_UP`.
pub fn tadpole_grow_up() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tadpole.grow_up"))
}

/// `SoundEvents.TADPOLE_HURT`.
pub fn tadpole_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tadpole.hurt"))
}

/// `SoundEvents.THORNS_HIT`.
pub fn thorns_hit() -> Holder<SoundEvent> {
    Holder::Key(key("enchant.thorns.hit"))
}

/// `SoundEvents.TNT_PRIMED`.
pub fn tnt_primed() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tnt.primed"))
}

/// `SoundEvents.TOTEM_USE`.
pub fn totem_use() -> Holder<SoundEvent> {
    Holder::Key(key("item.totem.use"))
}

/// `SoundEvents.TRIDENT_HIT`.
pub fn trident_hit() -> Holder<SoundEvent> {
    Holder::Key(key("item.trident.hit"))
}

/// `SoundEvents.TRIDENT_HIT_GROUND`.
pub fn trident_hit_ground() -> Holder<SoundEvent> {
    Holder::Key(key("item.trident.hit_ground"))
}

/// `SoundEvents.TRIDENT_RETURN`.
pub fn trident_return() -> Holder<SoundEvent> {
    Holder::Key(key("item.trident.return"))
}

/// `SoundEvents.TRIPWIRE_ATTACH`.
pub fn tripwire_attach() -> Holder<SoundEvent> {
    Holder::Key(key("block.tripwire.attach"))
}

/// `SoundEvents.TRIPWIRE_CLICK_OFF`.
pub fn tripwire_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.tripwire.click_off"))
}

/// `SoundEvents.TRIPWIRE_CLICK_ON`.
pub fn tripwire_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.tripwire.click_on"))
}

/// `SoundEvents.TRIPWIRE_DETACH`.
pub fn tripwire_detach() -> Holder<SoundEvent> {
    Holder::Key(key("block.tripwire.detach"))
}

/// `SoundEvents.TROPICAL_FISH_AMBIENT`.
pub fn tropical_fish_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tropical_fish.ambient"))
}

/// `SoundEvents.TROPICAL_FISH_DEATH`.
pub fn tropical_fish_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tropical_fish.death"))
}

/// `SoundEvents.TROPICAL_FISH_FLOP`.
pub fn tropical_fish_flop() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tropical_fish.flop"))
}

/// `SoundEvents.TROPICAL_FISH_HURT`.
pub fn tropical_fish_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.tropical_fish.hurt"))
}

/// `SoundEvents.TUFF_BREAK`.
pub fn tuff_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff.break"))
}

/// `SoundEvents.TUFF_STEP`.
pub fn tuff_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff.step"))
}

/// `SoundEvents.TUFF_PLACE`.
pub fn tuff_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff.place"))
}

/// `SoundEvents.TUFF_HIT`.
pub fn tuff_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff.hit"))
}

/// `SoundEvents.TUFF_FALL`.
pub fn tuff_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff.fall"))
}

/// `SoundEvents.TUFF_BRICKS_BREAK`.
pub fn tuff_bricks_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff_bricks.break"))
}

/// `SoundEvents.TUFF_BRICKS_FALL`.
pub fn tuff_bricks_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff_bricks.fall"))
}

/// `SoundEvents.TUFF_BRICKS_HIT`.
pub fn tuff_bricks_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff_bricks.hit"))
}

/// `SoundEvents.TUFF_BRICKS_PLACE`.
pub fn tuff_bricks_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff_bricks.place"))
}

/// `SoundEvents.TUFF_BRICKS_STEP`.
pub fn tuff_bricks_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.tuff_bricks.step"))
}

/// `SoundEvents.POLISHED_TUFF_BREAK`.
pub fn polished_tuff_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_tuff.break"))
}

/// `SoundEvents.POLISHED_TUFF_FALL`.
pub fn polished_tuff_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_tuff.fall"))
}

/// `SoundEvents.POLISHED_TUFF_HIT`.
pub fn polished_tuff_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_tuff.hit"))
}

/// `SoundEvents.POLISHED_TUFF_PLACE`.
pub fn polished_tuff_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_tuff.place"))
}

/// `SoundEvents.POLISHED_TUFF_STEP`.
pub fn polished_tuff_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.polished_tuff.step"))
}

/// `SoundEvents.TURTLE_AMBIENT_LAND`.
pub fn turtle_ambient_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.ambient_land"))
}

/// `SoundEvents.TURTLE_DEATH`.
pub fn turtle_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.death"))
}

/// `SoundEvents.TURTLE_DEATH_BABY`.
pub fn turtle_death_baby() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.death_baby"))
}

/// `SoundEvents.TURTLE_EGG_BREAK`.
pub fn turtle_egg_break() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.egg_break"))
}

/// `SoundEvents.TURTLE_EGG_CRACK`.
pub fn turtle_egg_crack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.egg_crack"))
}

/// `SoundEvents.TURTLE_EGG_HATCH`.
pub fn turtle_egg_hatch() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.egg_hatch"))
}

/// `SoundEvents.TURTLE_HURT`.
pub fn turtle_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.hurt"))
}

/// `SoundEvents.TURTLE_HURT_BABY`.
pub fn turtle_hurt_baby() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.hurt_baby"))
}

/// `SoundEvents.TURTLE_LAY_EGG`.
pub fn turtle_lay_egg() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.lay_egg"))
}

/// `SoundEvents.TURTLE_SHAMBLE`.
pub fn turtle_shamble() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.shamble"))
}

/// `SoundEvents.TURTLE_SHAMBLE_BABY`.
pub fn turtle_shamble_baby() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.shamble_baby"))
}

/// `SoundEvents.TURTLE_SWIM`.
pub fn turtle_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.turtle.swim"))
}

/// `SoundEvents.UI_BUTTON_CLICK`.
pub fn ui_button_click() -> Holder<SoundEvent> {
    Holder::Key(key("ui.button.click"))
}

/// `SoundEvents.UI_LOOM_SELECT_PATTERN`.
pub fn ui_loom_select_pattern() -> Holder<SoundEvent> {
    Holder::Key(key("ui.loom.select_pattern"))
}

/// `SoundEvents.UI_LOOM_TAKE_RESULT`.
pub fn ui_loom_take_result() -> Holder<SoundEvent> {
    Holder::Key(key("ui.loom.take_result"))
}

/// `SoundEvents.UI_CARTOGRAPHY_TABLE_TAKE_RESULT`.
pub fn ui_cartography_table_take_result() -> Holder<SoundEvent> {
    Holder::Key(key("ui.cartography_table.take_result"))
}

/// `SoundEvents.UI_STONECUTTER_TAKE_RESULT`.
pub fn ui_stonecutter_take_result() -> Holder<SoundEvent> {
    Holder::Key(key("ui.stonecutter.take_result"))
}

/// `SoundEvents.UI_STONECUTTER_SELECT_RECIPE`.
pub fn ui_stonecutter_select_recipe() -> Holder<SoundEvent> {
    Holder::Key(key("ui.stonecutter.select_recipe"))
}

/// `SoundEvents.UI_TOAST_CHALLENGE_COMPLETE`.
pub fn ui_toast_challenge_complete() -> Holder<SoundEvent> {
    Holder::Key(key("ui.toast.challenge_complete"))
}

/// `SoundEvents.UI_TOAST_IN`.
pub fn ui_toast_in() -> Holder<SoundEvent> {
    Holder::Key(key("ui.toast.in"))
}

/// `SoundEvents.UI_TOAST_OUT`.
pub fn ui_toast_out() -> Holder<SoundEvent> {
    Holder::Key(key("ui.toast.out"))
}

/// `SoundEvents.VAULT_ACTIVATE`.
pub fn vault_activate() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.activate"))
}

/// `SoundEvents.VAULT_AMBIENT`.
pub fn vault_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.ambient"))
}

/// `SoundEvents.VAULT_BREAK`.
pub fn vault_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.break"))
}

/// `SoundEvents.VAULT_CLOSE_SHUTTER`.
pub fn vault_close_shutter() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.close_shutter"))
}

/// `SoundEvents.VAULT_DEACTIVATE`.
pub fn vault_deactivate() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.deactivate"))
}

/// `SoundEvents.VAULT_EJECT_ITEM`.
pub fn vault_eject_item() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.eject_item"))
}

/// `SoundEvents.VAULT_REJECT_REWARDED_PLAYER`.
pub fn vault_reject_rewarded_player() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.reject_rewarded_player"))
}

/// `SoundEvents.VAULT_FALL`.
pub fn vault_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.fall"))
}

/// `SoundEvents.VAULT_HIT`.
pub fn vault_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.hit"))
}

/// `SoundEvents.VAULT_INSERT_ITEM`.
pub fn vault_insert_item() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.insert_item"))
}

/// `SoundEvents.VAULT_INSERT_ITEM_FAIL`.
pub fn vault_insert_item_fail() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.insert_item_fail"))
}

/// `SoundEvents.VAULT_OPEN_SHUTTER`.
pub fn vault_open_shutter() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.open_shutter"))
}

/// `SoundEvents.VAULT_PLACE`.
pub fn vault_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.place"))
}

/// `SoundEvents.VAULT_STEP`.
pub fn vault_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.vault.step"))
}

/// `SoundEvents.VEX_AMBIENT`.
pub fn vex_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.vex.ambient"))
}

/// `SoundEvents.VEX_CHARGE`.
pub fn vex_charge() -> Holder<SoundEvent> {
    Holder::Key(key("entity.vex.charge"))
}

/// `SoundEvents.VEX_DEATH`.
pub fn vex_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.vex.death"))
}

/// `SoundEvents.VEX_HURT`.
pub fn vex_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.vex.hurt"))
}

/// `SoundEvents.VILLAGER_AMBIENT`.
pub fn villager_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.ambient"))
}

/// `SoundEvents.VILLAGER_CELEBRATE`.
pub fn villager_celebrate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.celebrate"))
}

/// `SoundEvents.VILLAGER_DEATH`.
pub fn villager_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.death"))
}

/// `SoundEvents.VILLAGER_HURT`.
pub fn villager_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.hurt"))
}

/// `SoundEvents.VILLAGER_NO`.
pub fn villager_no() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.no"))
}

/// `SoundEvents.VILLAGER_TRADE`.
pub fn villager_trade() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.trade"))
}

/// `SoundEvents.VILLAGER_YES`.
pub fn villager_yes() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.yes"))
}

/// `SoundEvents.VILLAGER_WORK_ARMORER`.
pub fn villager_work_armorer() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_armorer"))
}

/// `SoundEvents.VILLAGER_WORK_BUTCHER`.
pub fn villager_work_butcher() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_butcher"))
}

/// `SoundEvents.VILLAGER_WORK_CARTOGRAPHER`.
pub fn villager_work_cartographer() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_cartographer"))
}

/// `SoundEvents.VILLAGER_WORK_CLERIC`.
pub fn villager_work_cleric() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_cleric"))
}

/// `SoundEvents.VILLAGER_WORK_FARMER`.
pub fn villager_work_farmer() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_farmer"))
}

/// `SoundEvents.VILLAGER_WORK_FISHERMAN`.
pub fn villager_work_fisherman() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_fisherman"))
}

/// `SoundEvents.VILLAGER_WORK_FLETCHER`.
pub fn villager_work_fletcher() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_fletcher"))
}

/// `SoundEvents.VILLAGER_WORK_LEATHERWORKER`.
pub fn villager_work_leatherworker() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_leatherworker"))
}

/// `SoundEvents.VILLAGER_WORK_LIBRARIAN`.
pub fn villager_work_librarian() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_librarian"))
}

/// `SoundEvents.VILLAGER_WORK_MASON`.
pub fn villager_work_mason() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_mason"))
}

/// `SoundEvents.VILLAGER_WORK_SHEPHERD`.
pub fn villager_work_shepherd() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_shepherd"))
}

/// `SoundEvents.VILLAGER_WORK_TOOLSMITH`.
pub fn villager_work_toolsmith() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_toolsmith"))
}

/// `SoundEvents.VILLAGER_WORK_WEAPONSMITH`.
pub fn villager_work_weaponsmith() -> Holder<SoundEvent> {
    Holder::Key(key("entity.villager.work_weaponsmith"))
}

/// `SoundEvents.VINDICATOR_AMBIENT`.
pub fn vindicator_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.vindicator.ambient"))
}

/// `SoundEvents.VINDICATOR_CELEBRATE`.
pub fn vindicator_celebrate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.vindicator.celebrate"))
}

/// `SoundEvents.VINDICATOR_DEATH`.
pub fn vindicator_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.vindicator.death"))
}

/// `SoundEvents.VINDICATOR_HURT`.
pub fn vindicator_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.vindicator.hurt"))
}

/// `SoundEvents.VINE_BREAK`.
pub fn vine_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.vine.break"))
}

/// `SoundEvents.VINE_FALL`.
pub fn vine_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.vine.fall"))
}

/// `SoundEvents.VINE_HIT`.
pub fn vine_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.vine.hit"))
}

/// `SoundEvents.VINE_PLACE`.
pub fn vine_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.vine.place"))
}

/// `SoundEvents.VINE_STEP`.
pub fn vine_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.vine.step"))
}

/// `SoundEvents.LILY_PAD_PLACE`.
pub fn lily_pad_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.lily_pad.place"))
}

/// `SoundEvents.WANDERING_TRADER_AMBIENT`.
pub fn wandering_trader_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.ambient"))
}

/// `SoundEvents.WANDERING_TRADER_DEATH`.
pub fn wandering_trader_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.death"))
}

/// `SoundEvents.WANDERING_TRADER_DISAPPEARED`.
pub fn wandering_trader_disappeared() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.disappeared"))
}

/// `SoundEvents.WANDERING_TRADER_DRINK_MILK`.
pub fn wandering_trader_drink_milk() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.drink_milk"))
}

/// `SoundEvents.WANDERING_TRADER_DRINK_POTION`.
pub fn wandering_trader_drink_potion() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.drink_potion"))
}

/// `SoundEvents.WANDERING_TRADER_HURT`.
pub fn wandering_trader_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.hurt"))
}

/// `SoundEvents.WANDERING_TRADER_NO`.
pub fn wandering_trader_no() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.no"))
}

/// `SoundEvents.WANDERING_TRADER_REAPPEARED`.
pub fn wandering_trader_reappeared() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.reappeared"))
}

/// `SoundEvents.WANDERING_TRADER_TRADE`.
pub fn wandering_trader_trade() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.trade"))
}

/// `SoundEvents.WANDERING_TRADER_YES`.
pub fn wandering_trader_yes() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wandering_trader.yes"))
}

/// `SoundEvents.WARDEN_AGITATED`.
pub fn warden_agitated() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.agitated"))
}

/// `SoundEvents.WARDEN_AMBIENT`.
pub fn warden_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.ambient"))
}

/// `SoundEvents.WARDEN_ANGRY`.
pub fn warden_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.angry"))
}

/// `SoundEvents.WARDEN_ATTACK_IMPACT`.
pub fn warden_attack_impact() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.attack_impact"))
}

/// `SoundEvents.WARDEN_DEATH`.
pub fn warden_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.death"))
}

/// `SoundEvents.WARDEN_DIG`.
pub fn warden_dig() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.dig"))
}

/// `SoundEvents.WARDEN_EMERGE`.
pub fn warden_emerge() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.emerge"))
}

/// `SoundEvents.WARDEN_HEARTBEAT`.
pub fn warden_heartbeat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.heartbeat"))
}

/// `SoundEvents.WARDEN_HURT`.
pub fn warden_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.hurt"))
}

/// `SoundEvents.WARDEN_LISTENING`.
pub fn warden_listening() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.listening"))
}

/// `SoundEvents.WARDEN_LISTENING_ANGRY`.
pub fn warden_listening_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.listening_angry"))
}

/// `SoundEvents.WARDEN_NEARBY_CLOSE`.
pub fn warden_nearby_close() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.nearby_close"))
}

/// `SoundEvents.WARDEN_NEARBY_CLOSER`.
pub fn warden_nearby_closer() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.nearby_closer"))
}

/// `SoundEvents.WARDEN_NEARBY_CLOSEST`.
pub fn warden_nearby_closest() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.nearby_closest"))
}

/// `SoundEvents.WARDEN_ROAR`.
pub fn warden_roar() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.roar"))
}

/// `SoundEvents.WARDEN_SNIFF`.
pub fn warden_sniff() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.sniff"))
}

/// `SoundEvents.WARDEN_SONIC_BOOM`.
pub fn warden_sonic_boom() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.sonic_boom"))
}

/// `SoundEvents.WARDEN_SONIC_CHARGE`.
pub fn warden_sonic_charge() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.sonic_charge"))
}

/// `SoundEvents.WARDEN_STEP`.
pub fn warden_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.step"))
}

/// `SoundEvents.WARDEN_TENDRIL_CLICKS`.
pub fn warden_tendril_clicks() -> Holder<SoundEvent> {
    Holder::Key(key("entity.warden.tendril_clicks"))
}

/// `SoundEvents.WAXED_HANGING_SIGN_INTERACT_FAIL`.
pub fn waxed_hanging_sign_interact_fail() -> Holder<SoundEvent> {
    Holder::Key(key("block.hanging_sign.waxed_interact_fail"))
}

/// `SoundEvents.WAXED_SIGN_INTERACT_FAIL`.
pub fn waxed_sign_interact_fail() -> Holder<SoundEvent> {
    Holder::Key(key("block.sign.waxed_interact_fail"))
}

/// `SoundEvents.WATER_AMBIENT`.
pub fn water_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("block.water.ambient"))
}

/// `SoundEvents.WEATHER_END_FLASH`.
pub fn weather_end_flash() -> Holder<SoundEvent> {
    Holder::Key(key("weather.end_flash"))
}

/// `SoundEvents.WEATHER_RAIN`.
pub fn weather_rain() -> Holder<SoundEvent> {
    Holder::Key(key("weather.rain"))
}

/// `SoundEvents.WEATHER_RAIN_ABOVE`.
pub fn weather_rain_above() -> Holder<SoundEvent> {
    Holder::Key(key("weather.rain.above"))
}

/// `SoundEvents.WET_GRASS_BREAK`.
pub fn wet_grass_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_grass.break"))
}

/// `SoundEvents.WET_GRASS_FALL`.
pub fn wet_grass_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_grass.fall"))
}

/// `SoundEvents.WET_GRASS_HIT`.
pub fn wet_grass_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_grass.hit"))
}

/// `SoundEvents.WET_GRASS_PLACE`.
pub fn wet_grass_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_grass.place"))
}

/// `SoundEvents.WET_GRASS_STEP`.
pub fn wet_grass_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_grass.step"))
}

/// `SoundEvents.WET_SPONGE_BREAK`.
pub fn wet_sponge_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_sponge.break"))
}

/// `SoundEvents.WET_SPONGE_DRIES`.
pub fn wet_sponge_dries() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_sponge.dries"))
}

/// `SoundEvents.WET_SPONGE_FALL`.
pub fn wet_sponge_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_sponge.fall"))
}

/// `SoundEvents.WET_SPONGE_HIT`.
pub fn wet_sponge_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_sponge.hit"))
}

/// `SoundEvents.WET_SPONGE_PLACE`.
pub fn wet_sponge_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_sponge.place"))
}

/// `SoundEvents.WET_SPONGE_STEP`.
pub fn wet_sponge_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.wet_sponge.step"))
}

/// `SoundEvents.WIND_CHARGE_BURST`.
pub fn wind_charge_burst() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wind_charge.wind_burst"))
}

/// `SoundEvents.WIND_CHARGE_THROW`.
pub fn wind_charge_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wind_charge.throw"))
}

/// `SoundEvents.WITCH_AMBIENT`.
pub fn witch_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.witch.ambient"))
}

/// `SoundEvents.WITCH_CELEBRATE`.
pub fn witch_celebrate() -> Holder<SoundEvent> {
    Holder::Key(key("entity.witch.celebrate"))
}

/// `SoundEvents.WITCH_DEATH`.
pub fn witch_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.witch.death"))
}

/// `SoundEvents.WITCH_DRINK`.
pub fn witch_drink() -> Holder<SoundEvent> {
    Holder::Key(key("entity.witch.drink"))
}

/// `SoundEvents.WITCH_HURT`.
pub fn witch_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.witch.hurt"))
}

/// `SoundEvents.WITCH_THROW`.
pub fn witch_throw() -> Holder<SoundEvent> {
    Holder::Key(key("entity.witch.throw"))
}

/// `SoundEvents.WITHER_AMBIENT`.
pub fn wither_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither.ambient"))
}

/// `SoundEvents.WITHER_BREAK_BLOCK`.
pub fn wither_break_block() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither.break_block"))
}

/// `SoundEvents.WITHER_DEATH`.
pub fn wither_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither.death"))
}

/// `SoundEvents.WITHER_HURT`.
pub fn wither_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither.hurt"))
}

/// `SoundEvents.WITHER_SHOOT`.
pub fn wither_shoot() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither.shoot"))
}

/// `SoundEvents.WITHER_SKELETON_AMBIENT`.
pub fn wither_skeleton_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither_skeleton.ambient"))
}

/// `SoundEvents.WITHER_SKELETON_DEATH`.
pub fn wither_skeleton_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither_skeleton.death"))
}

/// `SoundEvents.WITHER_SKELETON_HURT`.
pub fn wither_skeleton_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither_skeleton.hurt"))
}

/// `SoundEvents.WITHER_SKELETON_STEP`.
pub fn wither_skeleton_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither_skeleton.step"))
}

/// `SoundEvents.WITHER_SPAWN`.
pub fn wither_spawn() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wither.spawn"))
}

/// `SoundEvents.WOLF_ARMOR_BREAK`.
pub fn wolf_armor_break() -> Holder<SoundEvent> {
    Holder::Key(key("item.wolf_armor.break"))
}

/// `SoundEvents.WOLF_ARMOR_CRACK`.
pub fn wolf_armor_crack() -> Holder<SoundEvent> {
    Holder::Key(key("item.wolf_armor.crack"))
}

/// `SoundEvents.WOLF_ARMOR_DAMAGE`.
pub fn wolf_armor_damage() -> Holder<SoundEvent> {
    Holder::Key(key("item.wolf_armor.damage"))
}

/// `SoundEvents.WOLF_ARMOR_REPAIR`.
pub fn wolf_armor_repair() -> Holder<SoundEvent> {
    Holder::Key(key("item.wolf_armor.repair"))
}

/// `SoundEvents.WOLF_SHAKE`.
pub fn wolf_shake() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wolf.shake"))
}

/// `SoundEvents.WOLF_STEP`.
pub fn wolf_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.wolf.step"))
}

/// `SoundEvents.WOODEN_DOOR_CLOSE`.
pub fn wooden_door_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.wooden_door.close"))
}

/// `SoundEvents.WOODEN_DOOR_OPEN`.
pub fn wooden_door_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.wooden_door.open"))
}

/// `SoundEvents.WOODEN_TRAPDOOR_CLOSE`.
pub fn wooden_trapdoor_close() -> Holder<SoundEvent> {
    Holder::Key(key("block.wooden_trapdoor.close"))
}

/// `SoundEvents.WOODEN_TRAPDOOR_OPEN`.
pub fn wooden_trapdoor_open() -> Holder<SoundEvent> {
    Holder::Key(key("block.wooden_trapdoor.open"))
}

/// `SoundEvents.WOODEN_BUTTON_CLICK_OFF`.
pub fn wooden_button_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.wooden_button.click_off"))
}

/// `SoundEvents.WOODEN_BUTTON_CLICK_ON`.
pub fn wooden_button_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.wooden_button.click_on"))
}

/// `SoundEvents.WOODEN_PRESSURE_PLATE_CLICK_OFF`.
pub fn wooden_pressure_plate_click_off() -> Holder<SoundEvent> {
    Holder::Key(key("block.wooden_pressure_plate.click_off"))
}

/// `SoundEvents.WOODEN_PRESSURE_PLATE_CLICK_ON`.
pub fn wooden_pressure_plate_click_on() -> Holder<SoundEvent> {
    Holder::Key(key("block.wooden_pressure_plate.click_on"))
}

/// `SoundEvents.WOOD_BREAK`.
pub fn wood_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.wood.break"))
}

/// `SoundEvents.WOOD_FALL`.
pub fn wood_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.wood.fall"))
}

/// `SoundEvents.WOOD_HIT`.
pub fn wood_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.wood.hit"))
}

/// `SoundEvents.WOOD_PLACE`.
pub fn wood_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.wood.place"))
}

/// `SoundEvents.WOOD_STEP`.
pub fn wood_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.wood.step"))
}

/// `SoundEvents.WOOL_BREAK`.
pub fn wool_break() -> Holder<SoundEvent> {
    Holder::Key(key("block.wool.break"))
}

/// `SoundEvents.WOOL_FALL`.
pub fn wool_fall() -> Holder<SoundEvent> {
    Holder::Key(key("block.wool.fall"))
}

/// `SoundEvents.WOOL_HIT`.
pub fn wool_hit() -> Holder<SoundEvent> {
    Holder::Key(key("block.wool.hit"))
}

/// `SoundEvents.WOOL_PLACE`.
pub fn wool_place() -> Holder<SoundEvent> {
    Holder::Key(key("block.wool.place"))
}

/// `SoundEvents.WOOL_STEP`.
pub fn wool_step() -> Holder<SoundEvent> {
    Holder::Key(key("block.wool.step"))
}

/// `SoundEvents.ZOGLIN_AMBIENT`.
pub fn zoglin_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zoglin.ambient"))
}

/// `SoundEvents.ZOGLIN_ANGRY`.
pub fn zoglin_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zoglin.angry"))
}

/// `SoundEvents.ZOGLIN_ATTACK`.
pub fn zoglin_attack() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zoglin.attack"))
}

/// `SoundEvents.ZOGLIN_DEATH`.
pub fn zoglin_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zoglin.death"))
}

/// `SoundEvents.ZOGLIN_HURT`.
pub fn zoglin_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zoglin.hurt"))
}

/// `SoundEvents.ZOGLIN_STEP`.
pub fn zoglin_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zoglin.step"))
}

/// `SoundEvents.ZOMBIE_AMBIENT`.
pub fn zombie_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.ambient"))
}

/// `SoundEvents.ZOMBIE_ATTACK_WOODEN_DOOR`.
pub fn zombie_attack_wooden_door() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.attack_wooden_door"))
}

/// `SoundEvents.ZOMBIE_ATTACK_IRON_DOOR`.
pub fn zombie_attack_iron_door() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.attack_iron_door"))
}

/// `SoundEvents.ZOMBIE_BREAK_WOODEN_DOOR`.
pub fn zombie_break_wooden_door() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.break_wooden_door"))
}

/// `SoundEvents.ZOMBIE_CONVERTED_TO_DROWNED`.
pub fn zombie_converted_to_drowned() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.converted_to_drowned"))
}

/// `SoundEvents.ZOMBIE_DEATH`.
pub fn zombie_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.death"))
}

/// `SoundEvents.ZOMBIE_DESTROY_EGG`.
pub fn zombie_destroy_egg() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.destroy_egg"))
}

/// `SoundEvents.ZOMBIE_HORSE_AMBIENT`.
pub fn zombie_horse_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_horse.ambient"))
}

/// `SoundEvents.ZOMBIE_HORSE_ANGRY`.
pub fn zombie_horse_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_horse.angry"))
}

/// `SoundEvents.ZOMBIE_HORSE_DEATH`.
pub fn zombie_horse_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_horse.death"))
}

/// `SoundEvents.ZOMBIE_HORSE_EAT`.
pub fn zombie_horse_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_horse.eat"))
}

/// `SoundEvents.ZOMBIE_HORSE_HURT`.
pub fn zombie_horse_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_horse.hurt"))
}

/// `SoundEvents.ZOMBIE_HURT`.
pub fn zombie_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.hurt"))
}

/// `SoundEvents.ZOMBIE_INFECT`.
pub fn zombie_infect() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.infect"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_AMBIENT`.
pub fn zombie_nautilus_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.ambient"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_AMBIENT_ON_LAND`.
pub fn zombie_nautilus_ambient_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.ambient_land"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_DASH`.
pub fn zombie_nautilus_dash() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.dash"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_DASH_ON_LAND`.
pub fn zombie_nautilus_dash_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.dash_land"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_DASH_READY`.
pub fn zombie_nautilus_dash_ready() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.dash_ready"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_DASH_READY_ON_LAND`.
pub fn zombie_nautilus_dash_ready_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.dash_ready_land"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_DEATH`.
pub fn zombie_nautilus_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.death"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_DEATH_ON_LAND`.
pub fn zombie_nautilus_death_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.death_land"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_EAT`.
pub fn zombie_nautilus_eat() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.eat"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_HURT`.
pub fn zombie_nautilus_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.hurt"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_HURT_ON_LAND`.
pub fn zombie_nautilus_hurt_on_land() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.hurt_land"))
}

/// `SoundEvents.ZOMBIE_NAUTILUS_SWIM`.
pub fn zombie_nautilus_swim() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_nautilus.swim"))
}

/// `SoundEvents.ZOMBIFIED_PIGLIN_AMBIENT`.
pub fn zombified_piglin_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombified_piglin.ambient"))
}

/// `SoundEvents.ZOMBIFIED_PIGLIN_ANGRY`.
pub fn zombified_piglin_angry() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombified_piglin.angry"))
}

/// `SoundEvents.ZOMBIFIED_PIGLIN_DEATH`.
pub fn zombified_piglin_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombified_piglin.death"))
}

/// `SoundEvents.ZOMBIFIED_PIGLIN_HURT`.
pub fn zombified_piglin_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombified_piglin.hurt"))
}

/// `SoundEvents.ZOMBIE_STEP`.
pub fn zombie_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie.step"))
}

/// `SoundEvents.ZOMBIE_VILLAGER_AMBIENT`.
pub fn zombie_villager_ambient() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_villager.ambient"))
}

/// `SoundEvents.ZOMBIE_VILLAGER_CONVERTED`.
pub fn zombie_villager_converted() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_villager.converted"))
}

/// `SoundEvents.ZOMBIE_VILLAGER_CURE`.
pub fn zombie_villager_cure() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_villager.cure"))
}

/// `SoundEvents.ZOMBIE_VILLAGER_DEATH`.
pub fn zombie_villager_death() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_villager.death"))
}

/// `SoundEvents.ZOMBIE_VILLAGER_HURT`.
pub fn zombie_villager_hurt() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_villager.hurt"))
}

/// `SoundEvents.ZOMBIE_VILLAGER_STEP`.
pub fn zombie_villager_step() -> Holder<SoundEvent> {
    Holder::Key(key("entity.zombie_villager.step"))
}

/// `SoundEvents.APPLY_EFFECT_BAD_OMEN`.
pub fn apply_effect_bad_omen() -> Holder<SoundEvent> {
    Holder::Key(key("event.mob_effect.bad_omen"))
}

/// `SoundEvents.APPLY_EFFECT_TRIAL_OMEN`.
pub fn apply_effect_trial_omen() -> Holder<SoundEvent> {
    Holder::Key(key("event.mob_effect.trial_omen"))
}

/// `SoundEvents.APPLY_EFFECT_RAID_OMEN`.
pub fn apply_effect_raid_omen() -> Holder<SoundEvent> {
    Holder::Key(key("event.mob_effect.raid_omen"))
}

/// `SoundEvents.SADDLE_UNEQUIP`.
pub fn saddle_unequip() -> Holder<SoundEvent> {
    Holder::Key(key("item.saddle.unequip"))
}

/// `SoundEvents.NAUTILUS_SADDLE_UNDERWATER_EQUIP`.
pub fn nautilus_saddle_underwater_equip() -> Holder<SoundEvent> {
    Holder::Key(key("item.nautilus_saddle_underwater_equip"))
}

/// `SoundEvents.NAUTILUS_SADDLE_EQUIP`.
pub fn nautilus_saddle_equip() -> Holder<SoundEvent> {
    Holder::Key(key("item.nautilus_saddle_equip"))
}

/// Все registry-ключи звуков.
pub fn all_keys() -> Vec<ResourceKey<SoundEvent>> {
    vec![
        key("entity.allay.ambient_with_item"),
        key("entity.allay.ambient_without_item"),
        key("entity.allay.death"),
        key("entity.allay.hurt"),
        key("entity.allay.item_given"),
        key("entity.allay.item_taken"),
        key("entity.allay.item_thrown"),
        key("ambient.cave"),
        key("ambient.basalt_deltas.additions"),
        key("ambient.basalt_deltas.loop"),
        key("ambient.basalt_deltas.mood"),
        key("ambient.crimson_forest.additions"),
        key("ambient.crimson_forest.loop"),
        key("ambient.crimson_forest.mood"),
        key("ambient.nether_wastes.additions"),
        key("ambient.nether_wastes.loop"),
        key("ambient.nether_wastes.mood"),
        key("ambient.soul_sand_valley.additions"),
        key("ambient.soul_sand_valley.loop"),
        key("ambient.soul_sand_valley.mood"),
        key("ambient.warped_forest.additions"),
        key("ambient.warped_forest.loop"),
        key("ambient.warped_forest.mood"),
        key("ambient.underwater.enter"),
        key("ambient.underwater.exit"),
        key("ambient.underwater.loop"),
        key("ambient.underwater.loop.additions"),
        key("ambient.underwater.loop.additions.rare"),
        key("ambient.underwater.loop.additions.ultra_rare"),
        key("block.amethyst_block.break"),
        key("block.amethyst_block.chime"),
        key("block.amethyst_block.fall"),
        key("block.amethyst_block.hit"),
        key("block.amethyst_block.place"),
        key("block.amethyst_block.resonate"),
        key("block.amethyst_block.step"),
        key("block.amethyst_cluster.break"),
        key("block.amethyst_cluster.fall"),
        key("block.amethyst_cluster.hit"),
        key("block.amethyst_cluster.place"),
        key("block.amethyst_cluster.step"),
        key("block.ancient_debris.break"),
        key("block.ancient_debris.step"),
        key("block.ancient_debris.place"),
        key("block.ancient_debris.hit"),
        key("block.ancient_debris.fall"),
        key("block.anvil.break"),
        key("block.anvil.destroy"),
        key("block.anvil.fall"),
        key("block.anvil.hit"),
        key("block.anvil.land"),
        key("block.anvil.place"),
        key("block.anvil.step"),
        key("block.anvil.use"),
        key("entity.armadillo.eat"),
        key("entity.armadillo.hurt"),
        key("entity.armadillo.hurt_reduced"),
        key("entity.armadillo.ambient"),
        key("entity.armadillo.step"),
        key("entity.armadillo.death"),
        key("entity.armadillo.roll"),
        key("entity.armadillo.land"),
        key("entity.armadillo.scute_drop"),
        key("entity.armadillo.unroll_finish"),
        key("entity.armadillo.peek"),
        key("entity.armadillo.unroll_start"),
        key("entity.armadillo.brush"),
        key("item.armor.unequip_wolf"),
        key("item.armor.equip_nautilus"),
        key("item.armor.unequip_nautilus"),
        key("entity.armor_stand.break"),
        key("entity.armor_stand.fall"),
        key("entity.armor_stand.hit"),
        key("entity.armor_stand.place"),
        key("entity.arrow.hit"),
        key("entity.arrow.hit_player"),
        key("entity.arrow.shoot"),
        key("item.axe.strip"),
        key("item.axe.scrape"),
        key("item.axe.wax_off"),
        key("entity.axolotl.attack"),
        key("entity.axolotl.death"),
        key("entity.axolotl.hurt"),
        key("entity.axolotl.idle_air"),
        key("entity.axolotl.idle_water"),
        key("entity.axolotl.splash"),
        key("entity.axolotl.swim"),
        key("block.azalea.break"),
        key("block.azalea.fall"),
        key("block.azalea.hit"),
        key("block.azalea.place"),
        key("block.azalea.step"),
        key("block.azalea_leaves.break"),
        key("block.azalea_leaves.fall"),
        key("block.azalea_leaves.hit"),
        key("block.azalea_leaves.place"),
        key("block.azalea_leaves.step"),
        key("entity.baby_nautilus.ambient"),
        key("entity.baby_nautilus.ambient_land"),
        key("entity.baby_nautilus.death"),
        key("entity.baby_nautilus.death_land"),
        key("entity.baby_nautilus.eat"),
        key("entity.baby_nautilus.hurt"),
        key("entity.baby_nautilus.hurt_land"),
        key("entity.nautilus.riding"),
        key("entity.baby_nautilus.swim"),
        key("block.bamboo.break"),
        key("block.bamboo.fall"),
        key("block.bamboo.hit"),
        key("block.bamboo.place"),
        key("block.bamboo.step"),
        key("block.bamboo_sapling.break"),
        key("block.bamboo_sapling.hit"),
        key("block.bamboo_sapling.place"),
        key("block.bamboo_wood.break"),
        key("block.bamboo_wood.fall"),
        key("block.bamboo_wood.hit"),
        key("block.bamboo_wood.place"),
        key("block.bamboo_wood.step"),
        key("block.bamboo_wood_door.close"),
        key("block.bamboo_wood_door.open"),
        key("block.bamboo_wood_trapdoor.close"),
        key("block.bamboo_wood_trapdoor.open"),
        key("block.bamboo_wood_button.click_off"),
        key("block.bamboo_wood_button.click_on"),
        key("block.bamboo_wood_pressure_plate.click_off"),
        key("block.bamboo_wood_pressure_plate.click_on"),
        key("block.bamboo_wood_fence_gate.close"),
        key("block.bamboo_wood_fence_gate.open"),
        key("block.barrel.close"),
        key("block.barrel.open"),
        key("block.basalt.break"),
        key("block.basalt.step"),
        key("block.basalt.place"),
        key("block.basalt.hit"),
        key("block.basalt.fall"),
        key("entity.bat.ambient"),
        key("entity.bat.death"),
        key("entity.bat.hurt"),
        key("entity.bat.loop"),
        key("entity.bat.takeoff"),
        key("block.beacon.activate"),
        key("block.beacon.ambient"),
        key("block.beacon.deactivate"),
        key("block.beacon.power_select"),
        key("entity.bee.death"),
        key("entity.bee.hurt"),
        key("entity.bee.loop_aggressive"),
        key("entity.bee.loop"),
        key("entity.bee.sting"),
        key("entity.bee.pollinate"),
        key("block.beehive.drip"),
        key("block.beehive.enter"),
        key("block.beehive.exit"),
        key("block.beehive.shear"),
        key("block.beehive.work"),
        key("block.bell.use"),
        key("block.bell.resonate"),
        key("block.big_dripleaf.break"),
        key("block.big_dripleaf.fall"),
        key("block.big_dripleaf.hit"),
        key("block.big_dripleaf.place"),
        key("block.big_dripleaf.step"),
        key("entity.blaze.ambient"),
        key("entity.blaze.burn"),
        key("entity.blaze.death"),
        key("entity.blaze.hurt"),
        key("entity.blaze.shoot"),
        key("entity.boat.paddle_land"),
        key("entity.boat.paddle_water"),
        key("entity.bogged.ambient"),
        key("entity.bogged.death"),
        key("entity.bogged.hurt"),
        key("entity.bogged.shear"),
        key("entity.bogged.step"),
        key("block.bone_block.break"),
        key("block.bone_block.fall"),
        key("block.bone_block.hit"),
        key("block.bone_block.place"),
        key("block.bone_block.step"),
        key("item.bone_meal.use"),
        key("item.book.page_turn"),
        key("item.book.put"),
        key("block.blastfurnace.fire_crackle"),
        key("item.bottle.empty"),
        key("item.bottle.fill"),
        key("item.bottle.fill_dragonbreath"),
        key("entity.breeze.charge"),
        key("entity.breeze.deflect"),
        key("entity.breeze.inhale"),
        key("entity.breeze.idle_ground"),
        key("entity.breeze.idle_air"),
        key("entity.breeze.shoot"),
        key("entity.breeze.jump"),
        key("entity.breeze.land"),
        key("entity.breeze.slide"),
        key("entity.breeze.death"),
        key("entity.breeze.hurt"),
        key("entity.breeze.whirl"),
        key("entity.breeze.wind_burst"),
        key("block.brewing_stand.brew"),
        key("item.brush.brushing.generic"),
        key("item.brush.brushing.sand"),
        key("item.brush.brushing.gravel"),
        key("item.brush.brushing.sand.complete"),
        key("item.brush.brushing.gravel.complete"),
        key("block.bubble_column.bubble_pop"),
        key("block.bubble_column.upwards_ambient"),
        key("block.bubble_column.upwards_inside"),
        key("block.bubble_column.whirlpool_ambient"),
        key("block.bubble_column.whirlpool_inside"),
        key("ui.hud.bubble_pop"),
        key("item.bucket.empty"),
        key("item.bucket.empty_axolotl"),
        key("item.bucket.empty_fish"),
        key("item.bucket.empty_lava"),
        key("item.bucket.empty_powder_snow"),
        key("item.bucket.empty_tadpole"),
        key("item.bucket.fill"),
        key("item.bucket.fill_axolotl"),
        key("item.bucket.fill_fish"),
        key("item.bucket.fill_lava"),
        key("item.bucket.fill_powder_snow"),
        key("item.bucket.fill_tadpole"),
        key("item.bundle.drop_contents"),
        key("item.bundle.insert"),
        key("item.bundle.insert_fail"),
        key("item.bundle.remove_one"),
        key("block.cactus_flower.break"),
        key("block.cactus_flower.place"),
        key("block.cake.add_candle"),
        key("block.calcite.break"),
        key("block.calcite.step"),
        key("block.calcite.place"),
        key("block.calcite.hit"),
        key("block.calcite.fall"),
        key("entity.camel_husk.ambient"),
        key("entity.camel_husk.dash"),
        key("entity.camel_husk.dash_ready"),
        key("entity.camel_husk.death"),
        key("entity.camel_husk.eat"),
        key("entity.camel_husk.hurt"),
        key("entity.camel_husk.saddle"),
        key("entity.camel_husk.sit"),
        key("entity.camel_husk.stand"),
        key("entity.camel_husk.step"),
        key("entity.camel_husk.step_sand"),
        key("entity.camel.ambient"),
        key("entity.camel.dash"),
        key("entity.camel.dash_ready"),
        key("entity.camel.death"),
        key("entity.camel.eat"),
        key("entity.camel.hurt"),
        key("entity.camel.saddle"),
        key("entity.camel.sit"),
        key("entity.camel.stand"),
        key("entity.camel.step"),
        key("entity.camel.step_sand"),
        key("block.campfire.crackle"),
        key("block.candle.ambient"),
        key("block.candle.break"),
        key("block.candle.extinguish"),
        key("block.candle.fall"),
        key("block.candle.hit"),
        key("block.candle.place"),
        key("block.candle.step"),
        key("entity.cat.ambient"),
        key("entity.cat.stray_ambient"),
        key("entity.cat.death"),
        key("entity.cat.eat"),
        key("entity.cat.hiss"),
        key("entity.cat.beg_for_food"),
        key("entity.cat.hurt"),
        key("entity.cat.purr"),
        key("entity.cat.purreow"),
        key("block.cave_vines.break"),
        key("block.cave_vines.fall"),
        key("block.cave_vines.hit"),
        key("block.cave_vines.place"),
        key("block.cave_vines.step"),
        key("block.cave_vines.pick_berries"),
        key("block.chain.break"),
        key("block.chain.fall"),
        key("block.chain.hit"),
        key("block.chain.place"),
        key("block.chain.step"),
        key("block.cherry_wood.break"),
        key("block.cherry_wood.fall"),
        key("block.cherry_wood.hit"),
        key("block.cherry_wood.place"),
        key("block.cherry_wood.step"),
        key("block.cherry_sapling.break"),
        key("block.cherry_sapling.fall"),
        key("block.cherry_sapling.hit"),
        key("block.cherry_sapling.place"),
        key("block.cherry_sapling.step"),
        key("block.cherry_leaves.break"),
        key("block.cherry_leaves.fall"),
        key("block.cherry_leaves.hit"),
        key("block.cherry_leaves.place"),
        key("block.cherry_leaves.step"),
        key("block.cherry_wood_hanging_sign.step"),
        key("block.cherry_wood_hanging_sign.break"),
        key("block.cherry_wood_hanging_sign.fall"),
        key("block.cherry_wood_hanging_sign.hit"),
        key("block.cherry_wood_hanging_sign.place"),
        key("block.cherry_wood_door.close"),
        key("block.cherry_wood_door.open"),
        key("block.cherry_wood_trapdoor.close"),
        key("block.cherry_wood_trapdoor.open"),
        key("block.cherry_wood_button.click_off"),
        key("block.cherry_wood_button.click_on"),
        key("block.cherry_wood_pressure_plate.click_off"),
        key("block.cherry_wood_pressure_plate.click_on"),
        key("block.cherry_wood_fence_gate.close"),
        key("block.cherry_wood_fence_gate.open"),
        key("block.chest.close"),
        key("block.chest.locked"),
        key("block.chest.open"),
        key("entity.chicken.ambient"),
        key("entity.chicken.death"),
        key("entity.chicken.egg"),
        key("entity.chicken.hurt"),
        key("entity.chicken.step"),
        key("block.chiseled_bookshelf.break"),
        key("block.chiseled_bookshelf.fall"),
        key("block.chiseled_bookshelf.hit"),
        key("block.chiseled_bookshelf.insert"),
        key("block.chiseled_bookshelf.insert.enchanted"),
        key("block.chiseled_bookshelf.step"),
        key("block.chiseled_bookshelf.pickup"),
        key("block.chiseled_bookshelf.pickup.enchanted"),
        key("block.chiseled_bookshelf.place"),
        key("block.chorus_flower.death"),
        key("block.chorus_flower.grow"),
        key("item.chorus_fruit.teleport"),
        key("block.cobweb.break"),
        key("block.cobweb.step"),
        key("block.cobweb.place"),
        key("block.cobweb.hit"),
        key("block.cobweb.fall"),
        key("entity.cod.ambient"),
        key("entity.cod.death"),
        key("entity.cod.flop"),
        key("entity.cod.hurt"),
        key("block.comparator.click"),
        key("block.composter.empty"),
        key("block.composter.fill"),
        key("block.composter.fill_success"),
        key("block.composter.ready"),
        key("block.conduit.activate"),
        key("block.conduit.ambient"),
        key("block.conduit.ambient.short"),
        key("block.conduit.attack.target"),
        key("block.conduit.deactivate"),
        key("block.copper_bulb.break"),
        key("block.copper_bulb.step"),
        key("block.copper_bulb.place"),
        key("block.copper_bulb.hit"),
        key("block.copper_bulb.fall"),
        key("block.copper_bulb.turn_on"),
        key("block.copper_bulb.turn_off"),
        key("block.copper.break"),
        key("block.copper.step"),
        key("block.copper.place"),
        key("block.copper.hit"),
        key("block.copper.fall"),
        key("block.copper_chest.close"),
        key("block.copper_chest.open"),
        key("block.copper_chest_weathered.close"),
        key("block.copper_chest_weathered.open"),
        key("block.copper_chest_oxidized.close"),
        key("block.copper_chest_oxidized.open"),
        key("block.copper_door.close"),
        key("block.copper_door.open"),
        key("entity.copper_golem.step"),
        key("entity.copper_golem.hurt"),
        key("entity.copper_golem.death"),
        key("entity.copper_golem_weathered.step"),
        key("entity.copper_golem_weathered.hurt"),
        key("entity.copper_golem_weathered.death"),
        key("entity.copper_golem_oxidized.step"),
        key("entity.copper_golem_oxidized.hurt"),
        key("entity.copper_golem_oxidized.death"),
        key("entity.copper_golem.spin"),
        key("entity.copper_golem_weathered.spin"),
        key("entity.copper_golem_oxidized.spin"),
        key("entity.copper_golem.no_item_get"),
        key("entity.copper_golem.no_item_no_get"),
        key("entity.copper_golem.item_drop"),
        key("entity.copper_golem.item_no_drop"),
        key("entity.copper_golem_become_statue"),
        key("block.copper_golem_statue.break"),
        key("block.copper_golem_statue.place"),
        key("block.copper_golem_statue.hit"),
        key("block.copper_golem_statue.step"),
        key("block.copper_golem_statue.fall"),
        key("entity.copper_golem.spawn"),
        key("entity.copper_golem.shear"),
        key("block.copper_grate.break"),
        key("block.copper_grate.step"),
        key("block.copper_grate.place"),
        key("block.copper_grate.hit"),
        key("block.copper_grate.fall"),
        key("block.copper_trapdoor.close"),
        key("block.copper_trapdoor.open"),
        key("block.coral_block.break"),
        key("block.coral_block.fall"),
        key("block.coral_block.hit"),
        key("block.coral_block.place"),
        key("block.coral_block.step"),
        key("entity.cow.ambient"),
        key("entity.cow.death"),
        key("entity.cow.hurt"),
        key("entity.cow.milk"),
        key("entity.cow.step"),
        key("block.crafter.craft"),
        key("block.crafter.fail"),
        key("entity.creaking.ambient"),
        key("entity.creaking.activate"),
        key("entity.creaking.deactivate"),
        key("entity.creaking.attack"),
        key("entity.creaking.death"),
        key("entity.creaking.step"),
        key("entity.creaking.freeze"),
        key("entity.creaking.unfreeze"),
        key("entity.creaking.spawn"),
        key("entity.creaking.sway"),
        key("entity.creaking.twitch"),
        key("block.creaking_heart.break"),
        key("block.creaking_heart.fall"),
        key("block.creaking_heart.hit"),
        key("block.creaking_heart.hurt"),
        key("block.creaking_heart.place"),
        key("block.creaking_heart.step"),
        key("block.creaking_heart.idle"),
        key("block.creaking_heart.spawn"),
        key("entity.creeper.death"),
        key("entity.creeper.hurt"),
        key("entity.creeper.primed"),
        key("block.crop.break"),
        key("item.crop.plant"),
        key("item.crossbow.hit"),
        key("item.crossbow.shoot"),
        key("block.deadbush.idle"),
        key("block.decorated_pot.break"),
        key("block.decorated_pot.fall"),
        key("block.decorated_pot.hit"),
        key("block.decorated_pot.insert"),
        key("block.decorated_pot.insert_fail"),
        key("block.decorated_pot.step"),
        key("block.decorated_pot.place"),
        key("block.decorated_pot.shatter"),
        key("block.deepslate_bricks.break"),
        key("block.deepslate_bricks.fall"),
        key("block.deepslate_bricks.hit"),
        key("block.deepslate_bricks.place"),
        key("block.deepslate_bricks.step"),
        key("block.deepslate.break"),
        key("block.deepslate.fall"),
        key("block.deepslate.hit"),
        key("block.deepslate.place"),
        key("block.deepslate.step"),
        key("block.deepslate_tiles.break"),
        key("block.deepslate_tiles.fall"),
        key("block.deepslate_tiles.hit"),
        key("block.deepslate_tiles.place"),
        key("block.deepslate_tiles.step"),
        key("block.dispenser.dispense"),
        key("block.dispenser.fail"),
        key("block.dispenser.launch"),
        key("entity.dolphin.ambient"),
        key("entity.dolphin.ambient_water"),
        key("entity.dolphin.attack"),
        key("entity.dolphin.death"),
        key("entity.dolphin.eat"),
        key("entity.dolphin.hurt"),
        key("entity.dolphin.jump"),
        key("entity.dolphin.play"),
        key("entity.dolphin.splash"),
        key("entity.dolphin.swim"),
        key("entity.donkey.ambient"),
        key("entity.donkey.angry"),
        key("entity.donkey.chest"),
        key("entity.donkey.death"),
        key("entity.donkey.eat"),
        key("entity.donkey.hurt"),
        key("entity.donkey.jump"),
        key("block.dried_ghast.break"),
        key("block.dried_ghast.step"),
        key("block.dried_ghast.fall"),
        key("block.dried_ghast.ambient"),
        key("block.dried_ghast.ambient_water"),
        key("block.dried_ghast.place"),
        key("block.dried_ghast.place_in_water"),
        key("block.dried_ghast.transition"),
        key("block.dripstone_block.break"),
        key("block.dripstone_block.step"),
        key("block.dripstone_block.place"),
        key("block.dripstone_block.hit"),
        key("block.dripstone_block.fall"),
        key("block.dry_grass.ambient"),
        key("block.pointed_dripstone.break"),
        key("block.pointed_dripstone.step"),
        key("block.pointed_dripstone.place"),
        key("block.pointed_dripstone.hit"),
        key("block.pointed_dripstone.fall"),
        key("block.pointed_dripstone.land"),
        key("block.pointed_dripstone.drip_lava"),
        key("block.pointed_dripstone.drip_water"),
        key("block.pointed_dripstone.drip_lava_into_cauldron"),
        key("block.pointed_dripstone.drip_water_into_cauldron"),
        key("block.big_dripleaf.tilt_down"),
        key("block.big_dripleaf.tilt_up"),
        key("entity.drowned.ambient"),
        key("entity.drowned.ambient_water"),
        key("entity.drowned.death"),
        key("entity.drowned.death_water"),
        key("entity.drowned.hurt"),
        key("entity.drowned.hurt_water"),
        key("entity.drowned.shoot"),
        key("entity.drowned.step"),
        key("entity.drowned.swim"),
        key("item.dye.use"),
        key("entity.egg.throw"),
        key("entity.elder_guardian.ambient"),
        key("entity.elder_guardian.ambient_land"),
        key("entity.elder_guardian.curse"),
        key("entity.elder_guardian.death"),
        key("entity.elder_guardian.death_land"),
        key("entity.elder_guardian.flop"),
        key("entity.elder_guardian.hurt"),
        key("entity.elder_guardian.hurt_land"),
        key("item.elytra.flying"),
        key("block.enchantment_table.use"),
        key("block.ender_chest.close"),
        key("block.ender_chest.open"),
        key("entity.ender_dragon.ambient"),
        key("entity.ender_dragon.death"),
        key("entity.dragon_fireball.explode"),
        key("entity.ender_dragon.flap"),
        key("entity.ender_dragon.growl"),
        key("entity.ender_dragon.hurt"),
        key("entity.ender_dragon.shoot"),
        key("entity.ender_eye.death"),
        key("entity.ender_eye.launch"),
        key("entity.enderman.ambient"),
        key("entity.enderman.death"),
        key("entity.enderman.hurt"),
        key("entity.enderman.scream"),
        key("entity.enderman.stare"),
        key("entity.enderman.teleport"),
        key("entity.endermite.ambient"),
        key("entity.endermite.death"),
        key("entity.endermite.hurt"),
        key("entity.endermite.step"),
        key("entity.ender_pearl.throw"),
        key("block.end_gateway.spawn"),
        key("block.end_portal_frame.fill"),
        key("block.end_portal.spawn"),
        key("entity.evoker.ambient"),
        key("entity.evoker.cast_spell"),
        key("entity.evoker.celebrate"),
        key("entity.evoker.death"),
        key("entity.evoker_fangs.attack"),
        key("entity.evoker.hurt"),
        key("entity.evoker.prepare_attack"),
        key("entity.evoker.prepare_summon"),
        key("entity.evoker.prepare_wololo"),
        key("entity.experience_bottle.throw"),
        key("entity.experience_orb.pickup"),
        key("block.eyeblossom.open_long"),
        key("block.eyeblossom.open"),
        key("block.eyeblossom.close_long"),
        key("block.eyeblossom.close"),
        key("block.eyeblossom.idle"),
        key("block.fence_gate.close"),
        key("block.fence_gate.open"),
        key("item.firecharge.use"),
        key("block.firefly_bush.idle"),
        key("entity.firework_rocket.blast"),
        key("entity.firework_rocket.blast_far"),
        key("entity.firework_rocket.large_blast"),
        key("entity.firework_rocket.large_blast_far"),
        key("entity.firework_rocket.launch"),
        key("entity.firework_rocket.shoot"),
        key("entity.firework_rocket.twinkle"),
        key("entity.firework_rocket.twinkle_far"),
        key("block.fire.ambient"),
        key("block.fire.extinguish"),
        key("entity.fish.swim"),
        key("entity.fishing_bobber.retrieve"),
        key("entity.fishing_bobber.splash"),
        key("entity.fishing_bobber.throw"),
        key("item.flintandsteel.use"),
        key("block.flowering_azalea.break"),
        key("block.flowering_azalea.fall"),
        key("block.flowering_azalea.hit"),
        key("block.flowering_azalea.place"),
        key("block.flowering_azalea.step"),
        key("entity.fox.aggro"),
        key("entity.fox.ambient"),
        key("entity.fox.bite"),
        key("entity.fox.death"),
        key("entity.fox.eat"),
        key("entity.fox.hurt"),
        key("entity.fox.screech"),
        key("entity.fox.sleep"),
        key("entity.fox.sniff"),
        key("entity.fox.spit"),
        key("entity.fox.teleport"),
        key("block.suspicious_sand.break"),
        key("block.suspicious_sand.step"),
        key("block.suspicious_sand.place"),
        key("block.suspicious_sand.hit"),
        key("block.suspicious_sand.fall"),
        key("block.suspicious_gravel.break"),
        key("block.suspicious_gravel.step"),
        key("block.suspicious_gravel.place"),
        key("block.suspicious_gravel.hit"),
        key("block.suspicious_gravel.fall"),
        key("block.froglight.break"),
        key("block.froglight.fall"),
        key("block.froglight.hit"),
        key("block.froglight.place"),
        key("block.froglight.step"),
        key("block.frogspawn.step"),
        key("block.frogspawn.break"),
        key("block.frogspawn.fall"),
        key("block.frogspawn.hatch"),
        key("block.frogspawn.hit"),
        key("block.frogspawn.place"),
        key("entity.frog.ambient"),
        key("entity.frog.death"),
        key("entity.frog.eat"),
        key("entity.frog.hurt"),
        key("entity.frog.lay_spawn"),
        key("entity.frog.long_jump"),
        key("entity.frog.step"),
        key("entity.frog.tongue"),
        key("block.roots.break"),
        key("block.roots.step"),
        key("block.roots.place"),
        key("block.roots.hit"),
        key("block.roots.fall"),
        key("block.furnace.fire_crackle"),
        key("entity.generic.big_fall"),
        key("entity.generic.burn"),
        key("entity.generic.death"),
        key("entity.generic.drink"),
        key("entity.generic.eat"),
        key("entity.generic.explode"),
        key("entity.generic.extinguish_fire"),
        key("entity.generic.hurt"),
        key("entity.generic.small_fall"),
        key("entity.generic.splash"),
        key("entity.generic.swim"),
        key("entity.ghast.ambient"),
        key("entity.ghast.death"),
        key("entity.ghast.hurt"),
        key("entity.ghast.scream"),
        key("entity.ghast.shoot"),
        key("entity.ghast.warn"),
        key("entity.ghastling.ambient"),
        key("entity.ghastling.death"),
        key("entity.ghastling.hurt"),
        key("entity.ghastling.spawn"),
        key("block.gilded_blackstone.break"),
        key("block.gilded_blackstone.fall"),
        key("block.gilded_blackstone.hit"),
        key("block.gilded_blackstone.place"),
        key("block.gilded_blackstone.step"),
        key("block.glass.break"),
        key("block.glass.fall"),
        key("block.glass.hit"),
        key("block.glass.place"),
        key("block.glass.step"),
        key("item.glow_ink_sac.use"),
        key("entity.glow_item_frame.add_item"),
        key("entity.glow_item_frame.break"),
        key("entity.glow_item_frame.place"),
        key("entity.glow_item_frame.remove_item"),
        key("entity.glow_item_frame.rotate_item"),
        key("entity.glow_squid.ambient"),
        key("entity.glow_squid.death"),
        key("entity.glow_squid.hurt"),
        key("entity.glow_squid.squirt"),
        key("entity.goat.ambient"),
        key("entity.goat.death"),
        key("entity.goat.eat"),
        key("entity.goat.hurt"),
        key("entity.goat.long_jump"),
        key("entity.goat.milk"),
        key("entity.goat.prepare_ram"),
        key("entity.goat.ram_impact"),
        key("entity.goat.horn_break"),
        key("entity.goat.screaming.ambient"),
        key("entity.goat.screaming.death"),
        key("entity.goat.screaming.eat"),
        key("entity.goat.screaming.hurt"),
        key("entity.goat.screaming.long_jump"),
        key("entity.goat.screaming.milk"),
        key("entity.goat.screaming.prepare_ram"),
        key("entity.goat.screaming.ram_impact"),
        key("entity.goat.step"),
        key("block.grass.break"),
        key("block.grass.fall"),
        key("block.grass.hit"),
        key("block.grass.place"),
        key("block.grass.step"),
        key("block.gravel.break"),
        key("block.gravel.fall"),
        key("block.gravel.hit"),
        key("block.gravel.place"),
        key("block.gravel.step"),
        key("block.grindstone.use"),
        key("block.growing_plant.crop"),
        key("entity.guardian.ambient"),
        key("entity.guardian.ambient_land"),
        key("entity.guardian.attack"),
        key("entity.guardian.death"),
        key("entity.guardian.death_land"),
        key("entity.guardian.flop"),
        key("entity.guardian.hurt"),
        key("entity.guardian.hurt_land"),
        key("block.hanging_roots.break"),
        key("block.hanging_roots.fall"),
        key("block.hanging_roots.hit"),
        key("block.hanging_roots.place"),
        key("block.hanging_roots.step"),
        key("block.hanging_sign.step"),
        key("block.hanging_sign.break"),
        key("block.hanging_sign.fall"),
        key("block.hanging_sign.hit"),
        key("block.hanging_sign.place"),
        key("entity.happy_ghast.ambient"),
        key("entity.happy_ghast.death"),
        key("entity.happy_ghast.hurt"),
        key("entity.happy_ghast.riding"),
        key("block.heavy_core.break"),
        key("block.heavy_core.fall"),
        key("block.heavy_core.hit"),
        key("block.heavy_core.place"),
        key("block.heavy_core.step"),
        key("block.nether_wood_hanging_sign.step"),
        key("block.nether_wood_hanging_sign.break"),
        key("block.nether_wood_hanging_sign.fall"),
        key("block.nether_wood_hanging_sign.hit"),
        key("block.nether_wood_hanging_sign.place"),
        key("block.bamboo_wood_hanging_sign.step"),
        key("block.bamboo_wood_hanging_sign.break"),
        key("block.bamboo_wood_hanging_sign.fall"),
        key("block.bamboo_wood_hanging_sign.hit"),
        key("block.bamboo_wood_hanging_sign.place"),
        key("block.trial_spawner.break"),
        key("block.trial_spawner.step"),
        key("block.trial_spawner.place"),
        key("block.trial_spawner.hit"),
        key("block.trial_spawner.fall"),
        key("block.trial_spawner.spawn_mob"),
        key("block.trial_spawner.about_to_spawn_item"),
        key("block.trial_spawner.spawn_item"),
        key("block.trial_spawner.spawn_item_begin"),
        key("block.trial_spawner.detect_player"),
        key("block.trial_spawner.ominous_activate"),
        key("block.trial_spawner.ambient"),
        key("block.trial_spawner.ambient_ominous"),
        key("block.trial_spawner.open_shutter"),
        key("block.trial_spawner.close_shutter"),
        key("block.trial_spawner.eject_item"),
        key("entity.happy_ghast.equip"),
        key("entity.happy_ghast.unequip"),
        key("entity.happy_ghast.harness_goggles_up"),
        key("entity.happy_ghast.harness_goggles_down"),
        key("item.hoe.till"),
        key("entity.hoglin.ambient"),
        key("entity.hoglin.angry"),
        key("entity.hoglin.attack"),
        key("entity.hoglin.converted_to_zombified"),
        key("entity.hoglin.death"),
        key("entity.hoglin.hurt"),
        key("entity.hoglin.retreat"),
        key("entity.hoglin.step"),
        key("block.honey_block.break"),
        key("block.honey_block.fall"),
        key("block.honey_block.hit"),
        key("block.honey_block.place"),
        key("block.honey_block.slide"),
        key("block.honey_block.step"),
        key("item.honeycomb.wax_on"),
        key("item.honey_bottle.drink"),
        key("entity.horse.ambient"),
        key("entity.horse.angry"),
        key("entity.horse.armor"),
        key("item.horse_armor.unequip"),
        key("entity.horse.breathe"),
        key("entity.horse.death"),
        key("entity.horse.eat"),
        key("entity.horse.gallop"),
        key("entity.horse.hurt"),
        key("entity.horse.jump"),
        key("entity.horse.land"),
        key("entity.horse.saddle"),
        key("entity.horse.step"),
        key("entity.horse.step_wood"),
        key("entity.hostile.big_fall"),
        key("entity.hostile.death"),
        key("entity.hostile.hurt"),
        key("entity.hostile.small_fall"),
        key("entity.hostile.splash"),
        key("entity.hostile.swim"),
        key("entity.husk.ambient"),
        key("entity.husk.converted_to_zombie"),
        key("entity.husk.death"),
        key("entity.husk.hurt"),
        key("entity.husk.step"),
        key("entity.illusioner.ambient"),
        key("entity.illusioner.cast_spell"),
        key("entity.illusioner.death"),
        key("entity.illusioner.hurt"),
        key("entity.illusioner.mirror_move"),
        key("entity.illusioner.prepare_blindness"),
        key("entity.illusioner.prepare_mirror"),
        key("item.ink_sac.use"),
        key("block.iron.break"),
        key("block.iron.step"),
        key("block.iron.place"),
        key("block.iron.hit"),
        key("block.iron.fall"),
        key("block.iron_door.close"),
        key("block.iron_door.open"),
        key("entity.iron_golem.attack"),
        key("entity.iron_golem.damage"),
        key("entity.iron_golem.death"),
        key("entity.iron_golem.hurt"),
        key("entity.iron_golem.repair"),
        key("entity.iron_golem.step"),
        key("block.iron_trapdoor.close"),
        key("block.iron_trapdoor.open"),
        key("entity.item_frame.add_item"),
        key("entity.item_frame.break"),
        key("entity.item_frame.place"),
        key("entity.item_frame.remove_item"),
        key("entity.item_frame.rotate_item"),
        key("entity.item.break"),
        key("entity.item.pickup"),
        key("block.ladder.break"),
        key("block.ladder.fall"),
        key("block.ladder.hit"),
        key("block.ladder.place"),
        key("block.ladder.step"),
        key("block.lantern.break"),
        key("block.lantern.fall"),
        key("block.lantern.hit"),
        key("block.lantern.place"),
        key("block.lantern.step"),
        key("block.large_amethyst_bud.break"),
        key("block.large_amethyst_bud.place"),
        key("block.lava.ambient"),
        key("block.lava.extinguish"),
        key("block.lava.pop"),
        key("block.leaf_litter.break"),
        key("block.leaf_litter.step"),
        key("block.leaf_litter.place"),
        key("block.leaf_litter.hit"),
        key("block.leaf_litter.fall"),
        key("item.lead.untied"),
        key("item.lead.tied"),
        key("item.lead.break"),
        key("block.lever.click"),
        key("entity.lightning_bolt.impact"),
        key("entity.lightning_bolt.thunder"),
        key("entity.lingering_potion.throw"),
        key("entity.llama.ambient"),
        key("entity.llama.angry"),
        key("entity.llama.chest"),
        key("entity.llama.death"),
        key("entity.llama.eat"),
        key("entity.llama.hurt"),
        key("entity.llama.spit"),
        key("entity.llama.step"),
        key("entity.llama.swag"),
        key("item.llama_carpet.unequip"),
        key("entity.magma_cube.death_small"),
        key("block.lodestone.break"),
        key("block.lodestone.step"),
        key("block.lodestone.place"),
        key("block.lodestone.hit"),
        key("block.lodestone.fall"),
        key("item.lodestone_compass.lock"),
        key("item.mace.smash_air"),
        key("item.mace.smash_ground"),
        key("item.mace.smash_ground_heavy"),
        key("entity.magma_cube.death"),
        key("entity.magma_cube.hurt"),
        key("entity.magma_cube.hurt_small"),
        key("entity.magma_cube.jump"),
        key("entity.magma_cube.squish"),
        key("entity.magma_cube.squish_small"),
        key("block.mangrove_roots.break"),
        key("block.mangrove_roots.fall"),
        key("block.mangrove_roots.hit"),
        key("block.mangrove_roots.place"),
        key("block.mangrove_roots.step"),
        key("block.medium_amethyst_bud.break"),
        key("block.medium_amethyst_bud.place"),
        key("block.metal.break"),
        key("block.metal.fall"),
        key("block.metal.hit"),
        key("block.metal.place"),
        key("block.metal_pressure_plate.click_off"),
        key("block.metal_pressure_plate.click_on"),
        key("block.metal.step"),
        key("entity.minecart.inside.underwater"),
        key("entity.minecart.inside"),
        key("entity.minecart.riding"),
        key("entity.mooshroom.convert"),
        key("entity.mooshroom.eat"),
        key("entity.mooshroom.milk"),
        key("entity.mooshroom.suspicious_milk"),
        key("entity.mooshroom.shear"),
        key("block.moss_carpet.break"),
        key("block.moss_carpet.fall"),
        key("block.moss_carpet.hit"),
        key("block.moss_carpet.place"),
        key("block.moss_carpet.step"),
        key("block.pink_petals.break"),
        key("block.pink_petals.fall"),
        key("block.pink_petals.hit"),
        key("block.pink_petals.place"),
        key("block.pink_petals.step"),
        key("block.moss.break"),
        key("block.moss.fall"),
        key("block.moss.hit"),
        key("block.moss.place"),
        key("block.moss.step"),
        key("block.mud.break"),
        key("block.mud.fall"),
        key("block.mud.hit"),
        key("block.mud.place"),
        key("block.mud.step"),
        key("block.mud_bricks.break"),
        key("block.mud_bricks.fall"),
        key("block.mud_bricks.hit"),
        key("block.mud_bricks.place"),
        key("block.mud_bricks.step"),
        key("block.muddy_mangrove_roots.break"),
        key("block.muddy_mangrove_roots.fall"),
        key("block.muddy_mangrove_roots.hit"),
        key("block.muddy_mangrove_roots.place"),
        key("block.muddy_mangrove_roots.step"),
        key("entity.mule.ambient"),
        key("entity.mule.angry"),
        key("entity.mule.chest"),
        key("entity.mule.death"),
        key("entity.mule.eat"),
        key("entity.mule.hurt"),
        key("entity.mule.jump"),
        key("music.creative"),
        key("music.credits"),
        key("music_disc.5"),
        key("music_disc.11"),
        key("music_disc.13"),
        key("music_disc.blocks"),
        key("music_disc.cat"),
        key("music_disc.chirp"),
        key("music_disc.far"),
        key("music_disc.lava_chicken"),
        key("music_disc.mall"),
        key("music_disc.mellohi"),
        key("music_disc.pigstep"),
        key("music_disc.stal"),
        key("music_disc.strad"),
        key("music_disc.wait"),
        key("music_disc.ward"),
        key("music_disc.otherside"),
        key("music_disc.relic"),
        key("music_disc.creator"),
        key("music_disc.creator_music_box"),
        key("music_disc.precipice"),
        key("music_disc.tears"),
        key("music.dragon"),
        key("music.end"),
        key("music.game"),
        key("music.menu"),
        key("music.nether.basalt_deltas"),
        key("music.nether.crimson_forest"),
        key("music.overworld.deep_dark"),
        key("music.overworld.dripstone_caves"),
        key("music.overworld.grove"),
        key("music.overworld.jagged_peaks"),
        key("music.overworld.lush_caves"),
        key("music.overworld.swamp"),
        key("music.overworld.forest"),
        key("music.overworld.old_growth_taiga"),
        key("music.overworld.meadow"),
        key("music.overworld.cherry_grove"),
        key("music.nether.nether_wastes"),
        key("music.overworld.frozen_peaks"),
        key("music.overworld.snowy_slopes"),
        key("music.nether.soul_sand_valley"),
        key("music.overworld.stony_peaks"),
        key("music.nether.warped_forest"),
        key("music.overworld.flower_forest"),
        key("music.overworld.desert"),
        key("music.overworld.badlands"),
        key("music.overworld.jungle"),
        key("music.overworld.sparse_jungle"),
        key("music.overworld.bamboo_jungle"),
        key("music.under_water"),
        key("entity.nautilus.ambient"),
        key("entity.nautilus.ambient_land"),
        key("entity.nautilus.dash"),
        key("entity.nautilus.dash_land"),
        key("entity.nautilus.dash_ready"),
        key("entity.nautilus.dash_ready_land"),
        key("entity.nautilus.death"),
        key("entity.nautilus.death_land"),
        key("entity.nautilus.eat"),
        key("entity.nautilus.hurt"),
        key("entity.nautilus.hurt_land"),
        key("entity.nautilus.swim"),
        key("block.nether_bricks.break"),
        key("block.nether_bricks.step"),
        key("block.nether_bricks.place"),
        key("block.nether_bricks.hit"),
        key("block.nether_bricks.fall"),
        key("block.nether_wart.break"),
        key("item.nether_wart.plant"),
        key("block.nether_wood.break"),
        key("block.nether_wood.fall"),
        key("block.nether_wood.hit"),
        key("block.nether_wood.place"),
        key("block.nether_wood.step"),
        key("block.nether_wood_door.close"),
        key("block.nether_wood_door.open"),
        key("block.nether_wood_trapdoor.close"),
        key("block.nether_wood_trapdoor.open"),
        key("block.nether_wood_button.click_off"),
        key("block.nether_wood_button.click_on"),
        key("block.nether_wood_pressure_plate.click_off"),
        key("block.nether_wood_pressure_plate.click_on"),
        key("block.nether_wood_fence_gate.close"),
        key("block.nether_wood_fence_gate.open"),
        key("intentionally_empty"),
        key("block.packed_mud.break"),
        key("block.packed_mud.fall"),
        key("block.packed_mud.hit"),
        key("block.packed_mud.place"),
        key("block.packed_mud.step"),
        key("block.stem.break"),
        key("block.stem.step"),
        key("block.stem.place"),
        key("block.stem.hit"),
        key("block.stem.fall"),
        key("block.nylium.break"),
        key("block.nylium.step"),
        key("block.nylium.place"),
        key("block.nylium.hit"),
        key("block.nylium.fall"),
        key("block.nether_sprouts.break"),
        key("block.nether_sprouts.step"),
        key("block.nether_sprouts.place"),
        key("block.nether_sprouts.hit"),
        key("block.nether_sprouts.fall"),
        key("block.fungus.break"),
        key("block.fungus.step"),
        key("block.fungus.place"),
        key("block.fungus.hit"),
        key("block.fungus.fall"),
        key("block.weeping_vines.break"),
        key("block.weeping_vines.step"),
        key("block.weeping_vines.place"),
        key("block.weeping_vines.hit"),
        key("block.weeping_vines.fall"),
        key("block.wart_block.break"),
        key("block.wart_block.step"),
        key("block.wart_block.place"),
        key("block.wart_block.hit"),
        key("block.wart_block.fall"),
        key("block.netherite_block.break"),
        key("block.netherite_block.step"),
        key("block.netherite_block.place"),
        key("block.netherite_block.hit"),
        key("block.netherite_block.fall"),
        key("block.netherrack.break"),
        key("block.netherrack.step"),
        key("block.netherrack.place"),
        key("block.netherrack.hit"),
        key("block.netherrack.fall"),
        key("block.note_block.basedrum"),
        key("block.note_block.bass"),
        key("block.note_block.bell"),
        key("block.note_block.chime"),
        key("block.note_block.flute"),
        key("block.note_block.guitar"),
        key("block.note_block.harp"),
        key("block.note_block.hat"),
        key("block.note_block.pling"),
        key("block.note_block.snare"),
        key("block.note_block.xylophone"),
        key("block.note_block.iron_xylophone"),
        key("block.note_block.cow_bell"),
        key("block.note_block.didgeridoo"),
        key("block.note_block.bit"),
        key("block.note_block.banjo"),
        key("block.note_block.imitate.zombie"),
        key("block.note_block.imitate.skeleton"),
        key("block.note_block.imitate.creeper"),
        key("block.note_block.imitate.ender_dragon"),
        key("block.note_block.imitate.wither_skeleton"),
        key("block.note_block.imitate.piglin"),
        key("entity.ocelot.hurt"),
        key("entity.ocelot.ambient"),
        key("entity.ocelot.death"),
        key("item.ominous_bottle.dispose"),
        key("entity.painting.break"),
        key("entity.painting.place"),
        key("block.pale_hanging_moss.idle"),
        key("entity.panda.pre_sneeze"),
        key("entity.panda.sneeze"),
        key("entity.panda.ambient"),
        key("entity.panda.death"),
        key("entity.panda.eat"),
        key("entity.panda.step"),
        key("entity.panda.cant_breed"),
        key("entity.panda.aggressive_ambient"),
        key("entity.panda.worried_ambient"),
        key("entity.panda.hurt"),
        key("entity.panda.bite"),
        key("entity.parched.ambient"),
        key("entity.parched.death"),
        key("entity.parched.hurt"),
        key("entity.parched.step"),
        key("entity.parrot.ambient"),
        key("entity.parrot.death"),
        key("entity.parrot.eat"),
        key("entity.parrot.fly"),
        key("entity.parrot.hurt"),
        key("entity.parrot.imitate.blaze"),
        key("entity.parrot.imitate.bogged"),
        key("entity.parrot.imitate.breeze"),
        key("entity.parrot.imitate.camel_husk"),
        key("entity.parrot.imitate.creaking"),
        key("entity.parrot.imitate.creeper"),
        key("entity.parrot.imitate.drowned"),
        key("entity.parrot.imitate.elder_guardian"),
        key("entity.parrot.imitate.ender_dragon"),
        key("entity.parrot.imitate.endermite"),
        key("entity.parrot.imitate.evoker"),
        key("entity.parrot.imitate.ghast"),
        key("entity.parrot.imitate.guardian"),
        key("entity.parrot.imitate.hoglin"),
        key("entity.parrot.imitate.husk"),
        key("entity.parrot.imitate.illusioner"),
        key("entity.parrot.imitate.magma_cube"),
        key("entity.parrot.imitate.phantom"),
        key("entity.parrot.imitate.parched"),
        key("entity.parrot.imitate.piglin"),
        key("entity.parrot.imitate.piglin_brute"),
        key("entity.parrot.imitate.pillager"),
        key("entity.parrot.imitate.ravager"),
        key("entity.parrot.imitate.shulker"),
        key("entity.parrot.imitate.silverfish"),
        key("entity.parrot.imitate.skeleton"),
        key("entity.parrot.imitate.slime"),
        key("entity.parrot.imitate.spider"),
        key("entity.parrot.imitate.stray"),
        key("entity.parrot.imitate.vex"),
        key("entity.parrot.imitate.vindicator"),
        key("entity.parrot.imitate.warden"),
        key("entity.parrot.imitate.witch"),
        key("entity.parrot.imitate.wither"),
        key("entity.parrot.imitate.wither_skeleton"),
        key("entity.parrot.imitate.zoglin"),
        key("entity.parrot.imitate.zombie"),
        key("entity.parrot.imitate.zombie_horse"),
        key("entity.parrot.imitate.zombie_nautilus"),
        key("entity.parrot.imitate.zombie_villager"),
        key("entity.parrot.step"),
        key("entity.phantom.ambient"),
        key("entity.phantom.bite"),
        key("entity.phantom.death"),
        key("entity.phantom.flap"),
        key("entity.phantom.hurt"),
        key("entity.phantom.swoop"),
        key("entity.pig.ambient"),
        key("entity.pig.death"),
        key("entity.pig.hurt"),
        key("entity.pig.saddle"),
        key("entity.pig.step"),
        key("entity.piglin.admiring_item"),
        key("entity.piglin.ambient"),
        key("entity.piglin.angry"),
        key("entity.piglin.celebrate"),
        key("entity.piglin.death"),
        key("entity.piglin.jealous"),
        key("entity.piglin.hurt"),
        key("entity.piglin.retreat"),
        key("entity.piglin.step"),
        key("entity.piglin.converted_to_zombified"),
        key("entity.piglin_brute.ambient"),
        key("entity.piglin_brute.angry"),
        key("entity.piglin_brute.death"),
        key("entity.piglin_brute.hurt"),
        key("entity.piglin_brute.step"),
        key("entity.piglin_brute.converted_to_zombified"),
        key("entity.pillager.ambient"),
        key("entity.pillager.celebrate"),
        key("entity.pillager.death"),
        key("entity.pillager.hurt"),
        key("block.piston.contract"),
        key("block.piston.extend"),
        key("entity.player.attack.crit"),
        key("entity.player.attack.knockback"),
        key("entity.player.attack.nodamage"),
        key("entity.player.attack.strong"),
        key("entity.player.attack.sweep"),
        key("entity.player.attack.weak"),
        key("entity.player.big_fall"),
        key("entity.player.breath"),
        key("entity.player.burp"),
        key("entity.player.death"),
        key("entity.player.hurt"),
        key("entity.player.hurt_drown"),
        key("entity.player.hurt_freeze"),
        key("entity.player.hurt_on_fire"),
        key("entity.player.hurt_sweet_berry_bush"),
        key("entity.player.levelup"),
        key("entity.player.small_fall"),
        key("entity.player.splash"),
        key("entity.player.splash.high_speed"),
        key("entity.player.swim"),
        key("entity.player.teleport"),
        key("entity.polar_bear.ambient"),
        key("entity.polar_bear.ambient_baby"),
        key("entity.polar_bear.death"),
        key("entity.polar_bear.hurt"),
        key("entity.polar_bear.step"),
        key("entity.polar_bear.warning"),
        key("block.polished_deepslate.break"),
        key("block.polished_deepslate.fall"),
        key("block.polished_deepslate.hit"),
        key("block.polished_deepslate.place"),
        key("block.polished_deepslate.step"),
        key("block.portal.ambient"),
        key("block.portal.travel"),
        key("block.portal.trigger"),
        key("block.powder_snow.break"),
        key("block.powder_snow.fall"),
        key("block.powder_snow.hit"),
        key("block.powder_snow.place"),
        key("block.powder_snow.step"),
        key("entity.puffer_fish.blow_out"),
        key("entity.puffer_fish.blow_up"),
        key("entity.puffer_fish.death"),
        key("entity.puffer_fish.flop"),
        key("entity.puffer_fish.hurt"),
        key("entity.puffer_fish.sting"),
        key("block.pumpkin.carve"),
        key("entity.rabbit.ambient"),
        key("entity.rabbit.attack"),
        key("entity.rabbit.death"),
        key("entity.rabbit.hurt"),
        key("entity.rabbit.jump"),
        key("event.raid.horn"),
        key("entity.ravager.ambient"),
        key("entity.ravager.attack"),
        key("entity.ravager.celebrate"),
        key("entity.ravager.death"),
        key("entity.ravager.hurt"),
        key("entity.ravager.step"),
        key("entity.ravager.stunned"),
        key("entity.ravager.roar"),
        key("block.nether_gold_ore.break"),
        key("block.nether_gold_ore.fall"),
        key("block.nether_gold_ore.hit"),
        key("block.nether_gold_ore.place"),
        key("block.nether_gold_ore.step"),
        key("block.nether_ore.break"),
        key("block.nether_ore.fall"),
        key("block.nether_ore.hit"),
        key("block.nether_ore.place"),
        key("block.nether_ore.step"),
        key("block.redstone_torch.burnout"),
        key("block.resin.break"),
        key("block.resin.fall"),
        key("block.resin.place"),
        key("block.resin.step"),
        key("block.resin_bricks.break"),
        key("block.resin_bricks.fall"),
        key("block.resin_bricks.hit"),
        key("block.resin_bricks.place"),
        key("block.resin_bricks.step"),
        key("block.respawn_anchor.ambient"),
        key("block.respawn_anchor.charge"),
        key("block.respawn_anchor.deplete"),
        key("block.respawn_anchor.set_spawn"),
        key("block.rooted_dirt.break"),
        key("block.rooted_dirt.fall"),
        key("block.rooted_dirt.hit"),
        key("block.rooted_dirt.place"),
        key("block.rooted_dirt.step"),
        key("entity.salmon.ambient"),
        key("entity.salmon.death"),
        key("entity.salmon.flop"),
        key("entity.salmon.hurt"),
        key("block.sand.break"),
        key("block.sand.fall"),
        key("block.sand.hit"),
        key("block.sand.place"),
        key("block.sand.step"),
        key("block.sand.idle"),
        key("block.scaffolding.break"),
        key("block.scaffolding.fall"),
        key("block.scaffolding.hit"),
        key("block.scaffolding.place"),
        key("block.scaffolding.step"),
        key("block.sculk.spread"),
        key("block.sculk.charge"),
        key("block.sculk.break"),
        key("block.sculk.fall"),
        key("block.sculk.hit"),
        key("block.sculk.place"),
        key("block.sculk.step"),
        key("block.sculk_catalyst.bloom"),
        key("block.sculk_catalyst.break"),
        key("block.sculk_catalyst.fall"),
        key("block.sculk_catalyst.hit"),
        key("block.sculk_catalyst.place"),
        key("block.sculk_catalyst.step"),
        key("block.sculk_sensor.clicking"),
        key("block.sculk_sensor.clicking_stop"),
        key("block.sculk_sensor.break"),
        key("block.sculk_sensor.fall"),
        key("block.sculk_sensor.hit"),
        key("block.sculk_sensor.place"),
        key("block.sculk_sensor.step"),
        key("block.sculk_shrieker.break"),
        key("block.sculk_shrieker.fall"),
        key("block.sculk_shrieker.hit"),
        key("block.sculk_shrieker.place"),
        key("block.sculk_shrieker.shriek"),
        key("block.sculk_shrieker.step"),
        key("block.sculk_vein.break"),
        key("block.sculk_vein.fall"),
        key("block.sculk_vein.hit"),
        key("block.sculk_vein.place"),
        key("block.sculk_vein.step"),
        key("entity.sheep.ambient"),
        key("entity.sheep.death"),
        key("entity.sheep.hurt"),
        key("entity.sheep.shear"),
        key("entity.sheep.step"),
        key("item.shears.snip"),
        key("block.shelf.activate"),
        key("block.shelf.break"),
        key("block.shelf.deactivate"),
        key("block.shelf.fall"),
        key("block.shelf.hit"),
        key("block.shelf.multi_swap"),
        key("block.shelf.place"),
        key("block.shelf.place_item"),
        key("block.shelf.single_swap"),
        key("block.shelf.step"),
        key("block.shelf.take_item"),
        key("item.shield.block"),
        key("item.shield.break"),
        key("block.shroomlight.break"),
        key("block.shroomlight.step"),
        key("block.shroomlight.place"),
        key("block.shroomlight.hit"),
        key("block.shroomlight.fall"),
        key("item.shovel.flatten"),
        key("entity.shulker.ambient"),
        key("block.shulker_box.close"),
        key("block.shulker_box.open"),
        key("entity.shulker_bullet.hit"),
        key("entity.shulker_bullet.hurt"),
        key("entity.shulker.close"),
        key("entity.shulker.death"),
        key("entity.shulker.hurt"),
        key("entity.shulker.hurt_closed"),
        key("entity.shulker.open"),
        key("entity.shulker.shoot"),
        key("entity.shulker.teleport"),
        key("entity.silverfish.ambient"),
        key("entity.silverfish.death"),
        key("entity.silverfish.hurt"),
        key("entity.silverfish.step"),
        key("entity.skeleton.ambient"),
        key("entity.skeleton.converted_to_stray"),
        key("entity.skeleton.death"),
        key("entity.skeleton_horse.ambient"),
        key("entity.skeleton_horse.death"),
        key("entity.skeleton_horse.hurt"),
        key("entity.skeleton_horse.swim"),
        key("entity.skeleton_horse.ambient_water"),
        key("entity.skeleton_horse.gallop_water"),
        key("entity.skeleton_horse.jump_water"),
        key("entity.skeleton_horse.step_water"),
        key("entity.skeleton.hurt"),
        key("entity.skeleton.shoot"),
        key("entity.skeleton.step"),
        key("entity.slime.attack"),
        key("entity.slime.death"),
        key("entity.slime.hurt"),
        key("entity.slime.jump"),
        key("entity.slime.squish"),
        key("block.slime_block.break"),
        key("block.slime_block.fall"),
        key("block.slime_block.hit"),
        key("block.slime_block.place"),
        key("block.slime_block.step"),
        key("block.small_amethyst_bud.break"),
        key("block.small_amethyst_bud.place"),
        key("block.small_dripleaf.break"),
        key("block.small_dripleaf.fall"),
        key("block.small_dripleaf.hit"),
        key("block.small_dripleaf.place"),
        key("block.small_dripleaf.step"),
        key("block.soul_sand.break"),
        key("block.soul_sand.step"),
        key("block.soul_sand.place"),
        key("block.soul_sand.hit"),
        key("block.soul_sand.fall"),
        key("block.soul_soil.break"),
        key("block.soul_soil.step"),
        key("block.soul_soil.place"),
        key("block.soul_soil.hit"),
        key("block.soul_soil.fall"),
        key("particle.soul_escape"),
        key("block.spawner.break"),
        key("block.spawner.fall"),
        key("block.spawner.hit"),
        key("block.spawner.place"),
        key("block.spawner.step"),
        key("block.spore_blossom.break"),
        key("block.spore_blossom.fall"),
        key("block.spore_blossom.hit"),
        key("block.spore_blossom.place"),
        key("block.spore_blossom.step"),
        key("entity.strider.ambient"),
        key("entity.strider.happy"),
        key("entity.strider.retreat"),
        key("entity.strider.death"),
        key("entity.strider.hurt"),
        key("entity.strider.step"),
        key("entity.strider.step_lava"),
        key("entity.strider.eat"),
        key("entity.strider.saddle"),
        key("entity.slime.death_small"),
        key("entity.slime.hurt_small"),
        key("entity.slime.jump_small"),
        key("entity.slime.squish_small"),
        key("block.smithing_table.use"),
        key("block.smoker.smoke"),
        key("entity.sniffer.step"),
        key("entity.sniffer.eat"),
        key("entity.sniffer.idle"),
        key("entity.sniffer.hurt"),
        key("entity.sniffer.death"),
        key("entity.sniffer.drop_seed"),
        key("entity.sniffer.scenting"),
        key("entity.sniffer.sniffing"),
        key("entity.sniffer.searching"),
        key("entity.sniffer.digging"),
        key("entity.sniffer.digging_stop"),
        key("entity.sniffer.happy"),
        key("block.sniffer_egg.plop"),
        key("block.sniffer_egg.crack"),
        key("block.sniffer_egg.hatch"),
        key("entity.snowball.throw"),
        key("block.snow.break"),
        key("block.snow.fall"),
        key("entity.snow_golem.ambient"),
        key("entity.snow_golem.death"),
        key("entity.snow_golem.hurt"),
        key("entity.snow_golem.shoot"),
        key("entity.snow_golem.shear"),
        key("block.snow.hit"),
        key("block.snow.place"),
        key("block.snow.step"),
        key("entity.spider.ambient"),
        key("entity.spider.death"),
        key("entity.spider.hurt"),
        key("entity.spider.step"),
        key("entity.splash_potion.break"),
        key("entity.splash_potion.throw"),
        key("block.sponge.break"),
        key("block.sponge.fall"),
        key("block.sponge.hit"),
        key("block.sponge.place"),
        key("block.sponge.step"),
        key("block.sponge.absorb"),
        key("item.spyglass.use"),
        key("item.spyglass.stop_using"),
        key("entity.squid.ambient"),
        key("entity.squid.death"),
        key("entity.squid.hurt"),
        key("entity.squid.squirt"),
        key("block.stone.break"),
        key("block.stone_button.click_off"),
        key("block.stone_button.click_on"),
        key("block.stone.fall"),
        key("block.stone.hit"),
        key("block.stone.place"),
        key("block.stone_pressure_plate.click_off"),
        key("block.stone_pressure_plate.click_on"),
        key("block.stone.step"),
        key("entity.stray.ambient"),
        key("entity.stray.death"),
        key("entity.stray.hurt"),
        key("entity.stray.step"),
        key("block.sweet_berry_bush.break"),
        key("block.sweet_berry_bush.place"),
        key("block.sweet_berry_bush.pick_berries"),
        key("entity.tadpole.death"),
        key("entity.tadpole.flop"),
        key("entity.tadpole.grow_up"),
        key("entity.tadpole.hurt"),
        key("enchant.thorns.hit"),
        key("entity.tnt.primed"),
        key("item.totem.use"),
        key("item.trident.hit"),
        key("item.trident.hit_ground"),
        key("item.trident.return"),
        key("block.tripwire.attach"),
        key("block.tripwire.click_off"),
        key("block.tripwire.click_on"),
        key("block.tripwire.detach"),
        key("entity.tropical_fish.ambient"),
        key("entity.tropical_fish.death"),
        key("entity.tropical_fish.flop"),
        key("entity.tropical_fish.hurt"),
        key("block.tuff.break"),
        key("block.tuff.step"),
        key("block.tuff.place"),
        key("block.tuff.hit"),
        key("block.tuff.fall"),
        key("block.tuff_bricks.break"),
        key("block.tuff_bricks.fall"),
        key("block.tuff_bricks.hit"),
        key("block.tuff_bricks.place"),
        key("block.tuff_bricks.step"),
        key("block.polished_tuff.break"),
        key("block.polished_tuff.fall"),
        key("block.polished_tuff.hit"),
        key("block.polished_tuff.place"),
        key("block.polished_tuff.step"),
        key("entity.turtle.ambient_land"),
        key("entity.turtle.death"),
        key("entity.turtle.death_baby"),
        key("entity.turtle.egg_break"),
        key("entity.turtle.egg_crack"),
        key("entity.turtle.egg_hatch"),
        key("entity.turtle.hurt"),
        key("entity.turtle.hurt_baby"),
        key("entity.turtle.lay_egg"),
        key("entity.turtle.shamble"),
        key("entity.turtle.shamble_baby"),
        key("entity.turtle.swim"),
        key("ui.button.click"),
        key("ui.loom.select_pattern"),
        key("ui.loom.take_result"),
        key("ui.cartography_table.take_result"),
        key("ui.stonecutter.take_result"),
        key("ui.stonecutter.select_recipe"),
        key("ui.toast.challenge_complete"),
        key("ui.toast.in"),
        key("ui.toast.out"),
        key("block.vault.activate"),
        key("block.vault.ambient"),
        key("block.vault.break"),
        key("block.vault.close_shutter"),
        key("block.vault.deactivate"),
        key("block.vault.eject_item"),
        key("block.vault.reject_rewarded_player"),
        key("block.vault.fall"),
        key("block.vault.hit"),
        key("block.vault.insert_item"),
        key("block.vault.insert_item_fail"),
        key("block.vault.open_shutter"),
        key("block.vault.place"),
        key("block.vault.step"),
        key("entity.vex.ambient"),
        key("entity.vex.charge"),
        key("entity.vex.death"),
        key("entity.vex.hurt"),
        key("entity.villager.ambient"),
        key("entity.villager.celebrate"),
        key("entity.villager.death"),
        key("entity.villager.hurt"),
        key("entity.villager.no"),
        key("entity.villager.trade"),
        key("entity.villager.yes"),
        key("entity.villager.work_armorer"),
        key("entity.villager.work_butcher"),
        key("entity.villager.work_cartographer"),
        key("entity.villager.work_cleric"),
        key("entity.villager.work_farmer"),
        key("entity.villager.work_fisherman"),
        key("entity.villager.work_fletcher"),
        key("entity.villager.work_leatherworker"),
        key("entity.villager.work_librarian"),
        key("entity.villager.work_mason"),
        key("entity.villager.work_shepherd"),
        key("entity.villager.work_toolsmith"),
        key("entity.villager.work_weaponsmith"),
        key("entity.vindicator.ambient"),
        key("entity.vindicator.celebrate"),
        key("entity.vindicator.death"),
        key("entity.vindicator.hurt"),
        key("block.vine.break"),
        key("block.vine.fall"),
        key("block.vine.hit"),
        key("block.vine.place"),
        key("block.vine.step"),
        key("block.lily_pad.place"),
        key("entity.wandering_trader.ambient"),
        key("entity.wandering_trader.death"),
        key("entity.wandering_trader.disappeared"),
        key("entity.wandering_trader.drink_milk"),
        key("entity.wandering_trader.drink_potion"),
        key("entity.wandering_trader.hurt"),
        key("entity.wandering_trader.no"),
        key("entity.wandering_trader.reappeared"),
        key("entity.wandering_trader.trade"),
        key("entity.wandering_trader.yes"),
        key("entity.warden.agitated"),
        key("entity.warden.ambient"),
        key("entity.warden.angry"),
        key("entity.warden.attack_impact"),
        key("entity.warden.death"),
        key("entity.warden.dig"),
        key("entity.warden.emerge"),
        key("entity.warden.heartbeat"),
        key("entity.warden.hurt"),
        key("entity.warden.listening"),
        key("entity.warden.listening_angry"),
        key("entity.warden.nearby_close"),
        key("entity.warden.nearby_closer"),
        key("entity.warden.nearby_closest"),
        key("entity.warden.roar"),
        key("entity.warden.sniff"),
        key("entity.warden.sonic_boom"),
        key("entity.warden.sonic_charge"),
        key("entity.warden.step"),
        key("entity.warden.tendril_clicks"),
        key("block.hanging_sign.waxed_interact_fail"),
        key("block.sign.waxed_interact_fail"),
        key("block.water.ambient"),
        key("weather.end_flash"),
        key("weather.rain"),
        key("weather.rain.above"),
        key("block.wet_grass.break"),
        key("block.wet_grass.fall"),
        key("block.wet_grass.hit"),
        key("block.wet_grass.place"),
        key("block.wet_grass.step"),
        key("block.wet_sponge.break"),
        key("block.wet_sponge.dries"),
        key("block.wet_sponge.fall"),
        key("block.wet_sponge.hit"),
        key("block.wet_sponge.place"),
        key("block.wet_sponge.step"),
        key("entity.wind_charge.wind_burst"),
        key("entity.wind_charge.throw"),
        key("entity.witch.ambient"),
        key("entity.witch.celebrate"),
        key("entity.witch.death"),
        key("entity.witch.drink"),
        key("entity.witch.hurt"),
        key("entity.witch.throw"),
        key("entity.wither.ambient"),
        key("entity.wither.break_block"),
        key("entity.wither.death"),
        key("entity.wither.hurt"),
        key("entity.wither.shoot"),
        key("entity.wither_skeleton.ambient"),
        key("entity.wither_skeleton.death"),
        key("entity.wither_skeleton.hurt"),
        key("entity.wither_skeleton.step"),
        key("entity.wither.spawn"),
        key("item.wolf_armor.break"),
        key("item.wolf_armor.crack"),
        key("item.wolf_armor.damage"),
        key("item.wolf_armor.repair"),
        key("entity.wolf.shake"),
        key("entity.wolf.step"),
        key("block.wooden_door.close"),
        key("block.wooden_door.open"),
        key("block.wooden_trapdoor.close"),
        key("block.wooden_trapdoor.open"),
        key("block.wooden_button.click_off"),
        key("block.wooden_button.click_on"),
        key("block.wooden_pressure_plate.click_off"),
        key("block.wooden_pressure_plate.click_on"),
        key("block.wood.break"),
        key("block.wood.fall"),
        key("block.wood.hit"),
        key("block.wood.place"),
        key("block.wood.step"),
        key("block.wool.break"),
        key("block.wool.fall"),
        key("block.wool.hit"),
        key("block.wool.place"),
        key("block.wool.step"),
        key("entity.zoglin.ambient"),
        key("entity.zoglin.angry"),
        key("entity.zoglin.attack"),
        key("entity.zoglin.death"),
        key("entity.zoglin.hurt"),
        key("entity.zoglin.step"),
        key("entity.zombie.ambient"),
        key("entity.zombie.attack_wooden_door"),
        key("entity.zombie.attack_iron_door"),
        key("entity.zombie.break_wooden_door"),
        key("entity.zombie.converted_to_drowned"),
        key("entity.zombie.death"),
        key("entity.zombie.destroy_egg"),
        key("entity.zombie_horse.ambient"),
        key("entity.zombie_horse.angry"),
        key("entity.zombie_horse.death"),
        key("entity.zombie_horse.eat"),
        key("entity.zombie_horse.hurt"),
        key("entity.zombie.hurt"),
        key("entity.zombie.infect"),
        key("entity.zombie_nautilus.ambient"),
        key("entity.zombie_nautilus.ambient_land"),
        key("entity.zombie_nautilus.dash"),
        key("entity.zombie_nautilus.dash_land"),
        key("entity.zombie_nautilus.dash_ready"),
        key("entity.zombie_nautilus.dash_ready_land"),
        key("entity.zombie_nautilus.death"),
        key("entity.zombie_nautilus.death_land"),
        key("entity.zombie_nautilus.eat"),
        key("entity.zombie_nautilus.hurt"),
        key("entity.zombie_nautilus.hurt_land"),
        key("entity.zombie_nautilus.swim"),
        key("entity.zombified_piglin.ambient"),
        key("entity.zombified_piglin.angry"),
        key("entity.zombified_piglin.death"),
        key("entity.zombified_piglin.hurt"),
        key("entity.zombie.step"),
        key("entity.zombie_villager.ambient"),
        key("entity.zombie_villager.converted"),
        key("entity.zombie_villager.cure"),
        key("entity.zombie_villager.death"),
        key("entity.zombie_villager.hurt"),
        key("entity.zombie_villager.step"),
        key("event.mob_effect.bad_omen"),
        key("event.mob_effect.trial_omen"),
        key("event.mob_effect.raid_omen"),
        key("item.saddle.unequip"),
        key("item.nautilus_saddle_underwater_equip"),
        key("item.nautilus_saddle_equip"),
    ]
}
