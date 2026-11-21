//! Последовательность загрузки и генерации для запроса чанка.

use crate::world::level::chunk::chunk_pyramid::{
    ChunkPyramid, ChunkStep, GENERATION_PYRAMID, LOADING_PYRAMID,
};
use crate::world::level::chunk::level_chunk::ChunkStatus;
use crate::world::level::chunk_pos::ChunkPos;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkStepMode {
    Load,
    Generate,
}

impl ChunkStepMode {
    fn pyramid(self) -> &'static ChunkPyramid {
        match self {
            Self::Load => &LOADING_PYRAMID,
            Self::Generate => &GENERATION_PYRAMID,
        }
    }
}

/// Доступ к сохранённым данным и завершённым стадиям чанков в планировщике.
pub trait GeneratingChunkMap {
    type Error;

    fn persisted_status(&self, pos: ChunkPos) -> Option<ChunkStatus>;
    fn completed_status(&self, pos: ChunkPos) -> Option<ChunkStatus>;

    /// После успешной стадии карта публикует её как завершённую.
    /// На стадии Empty карта читает диск или создаёт пустой ProtoChunk.
    fn apply_step(
        &mut self,
        pos: ChunkPos,
        step: &ChunkStep,
        mode: ChunkStepMode,
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy, Debug)]
pub struct ChunkGenerationTask {
    pub center: ChunkPos,
    pub target_status: ChunkStatus,
}

impl ChunkGenerationTask {
    /// Выполняет стадии по порядку, заканчивая каждую группу чанков перед следующей.
    pub fn run<M: GeneratingChunkMap>(&self, map: &mut M) -> Result<(), M::Error> {
        self.run_step(map, self.center, ChunkStatus::Empty, ChunkStepMode::Load)?;
        if self.target_status == ChunkStatus::Empty {
            return Ok(());
        }

        let load = ChunkStepMode::Load;
        let mode = if self.can_load_without_generation(map) {
            load
        } else {
            ChunkStepMode::Generate
        };
        let target = mode.pyramid().get_step_to(self.target_status);
        let last = self.target_status.ordinal();
        for &status in &ChunkStatus::VALUES[..=last] {
            let radius = target.accumulated_radius_of(status).unwrap();
            for pos in chunks_within(self.center, radius) {
                let step_mode = if status == ChunkStatus::Empty
                    || map
                        .persisted_status(pos)
                        .is_some_and(|persisted| persisted.is_or_after(status))
                {
                    load
                } else {
                    ChunkStepMode::Generate
                };
                self.run_step(map, pos, status, step_mode)?;
            }
        }
        Ok(())
    }

    fn run_step<M: GeneratingChunkMap>(
        &self,
        map: &mut M,
        pos: ChunkPos,
        status: ChunkStatus,
        mode: ChunkStepMode,
    ) -> Result<(), M::Error> {
        if map
            .completed_status(pos)
            .is_some_and(|completed| completed.is_or_after(status))
        {
            return Ok(());
        }
        map.apply_step(pos, mode.pyramid().get_step_to(status), mode)
    }

    fn can_load_without_generation(&self, map: &impl GeneratingChunkMap) -> bool {
        if !map
            .persisted_status(self.center)
            .is_some_and(|status| status.is_or_after(self.target_status))
        {
            return false;
        }
        let target = LOADING_PYRAMID.get_step_to(self.target_status);
        let radius = target.accumulated_radius_of(ChunkStatus::Empty).unwrap();
        chunks_within(self.center, radius).all(|pos| {
            let distance = self
                .center
                .x
                .abs_diff(pos.x)
                .max(self.center.z.abs_diff(pos.z));
            let required = target.accumulated_dependencies.at_radius(distance as usize);
            required.is_some_and(|required| {
                map.persisted_status(pos)
                    .is_some_and(|status| status.is_or_after(required))
            })
        })
    }
}

fn chunks_within(center: ChunkPos, radius: usize) -> impl Iterator<Item = ChunkPos> {
    let radius = radius as i32;
    (-radius..=radius).flat_map(move |dx| {
        (-radius..=radius)
            .map(move |dz| ChunkPos::new(center.x.wrapping_add(dx), center.z.wrapping_add(dz)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Default)]
    struct ChunkMap {
        on_disk: HashMap<ChunkPos, ChunkStatus>,
        persisted: HashMap<ChunkPos, ChunkStatus>,
        completed: HashMap<ChunkPos, ChunkStatus>,
        steps: Vec<(ChunkPos, ChunkStatus, ChunkStepMode)>,
        fail_at: Option<ChunkStatus>,
    }

    impl GeneratingChunkMap for ChunkMap {
        type Error = ChunkStatus;

        fn persisted_status(&self, pos: ChunkPos) -> Option<ChunkStatus> {
            self.persisted.get(&pos).copied()
        }

        fn completed_status(&self, pos: ChunkPos) -> Option<ChunkStatus> {
            self.completed.get(&pos).copied()
        }

        fn apply_step(
            &mut self,
            pos: ChunkPos,
            step: &ChunkStep,
            mode: ChunkStepMode,
        ) -> Result<(), Self::Error> {
            let status = step.target_status;
            if self.fail_at == Some(status) {
                return Err(status);
            }
            if status == ChunkStatus::Empty {
                self.persisted
                    .insert(pos, self.on_disk.get(&pos).copied().unwrap_or(status));
            } else if mode == ChunkStepMode::Generate {
                self.persisted.insert(pos, status);
            }
            self.completed.insert(pos, status);
            self.steps.push((pos, status, mode));
            Ok(())
        }
    }

    #[test]
    fn loaded_chunks_skip_neighbour_generation() {
        let center = ChunkPos::new(-1, 5);
        let mut map = ChunkMap::default();
        map.on_disk.insert(center, ChunkStatus::Full);
        let task = ChunkGenerationTask {
            center,
            target_status: ChunkStatus::Full,
        };
        task.run(&mut map).unwrap();
        assert_eq!(map.steps.len(), ChunkStatus::VALUES.len());
        assert!(map
            .steps
            .iter()
            .all(|(pos, _, mode)| *pos == center && *mode == ChunkStepMode::Load));
    }

    #[test]
    fn generation_completes_neighbour_layers_and_retries_only_pending_work() {
        let center = ChunkPos::new(0, 0);
        let outer = ChunkPos::new(8, 8);
        let mut map = ChunkMap {
            fail_at: Some(ChunkStatus::StructureReferences),
            ..ChunkMap::default()
        };
        let task = ChunkGenerationTask {
            center,
            target_status: ChunkStatus::StructureReferences,
        };
        assert_eq!(task.run(&mut map), Err(ChunkStatus::StructureReferences));
        assert_eq!(
            map.completed_status(outer),
            Some(ChunkStatus::StructureStarts)
        );

        let already_completed = map.steps.len();
        map.fail_at = None;
        task.run(&mut map).unwrap();
        assert_eq!(
            &map.steps[already_completed..],
            &[(
                center,
                ChunkStatus::StructureReferences,
                ChunkStepMode::Generate
            )]
        );
        assert_eq!(
            map.completed_status(center),
            Some(ChunkStatus::StructureReferences)
        );
    }
}
