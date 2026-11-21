//! Пирамиды зависимостей стадий загрузки и генерации чанков.

use std::sync::LazyLock;

use super::level_chunk::ChunkStatus::{self, *};

/// Минимальная стадия соседних чанков по радиусу Чебышёва.
#[derive(Debug)]
pub struct ChunkDependencies {
    by_radius: Vec<ChunkStatus>,
}

impl ChunkDependencies {
    pub fn as_slice(&self) -> &[ChunkStatus] {
        &self.by_radius
    }

    pub fn at_radius(&self, radius: usize) -> Option<ChunkStatus> {
        self.by_radius.get(radius).copied()
    }

    pub fn radius_of(&self, status: ChunkStatus) -> Option<usize> {
        self.by_radius
            .iter()
            .rposition(|required| required.is_or_after(status))
    }
}

/// Требования к одной стадии и радиус записи блоков из WorldGenRegion.
#[derive(Debug)]
pub struct ChunkStep {
    pub target_status: ChunkStatus,
    pub direct_dependencies: ChunkDependencies,
    pub accumulated_dependencies: ChunkDependencies,
    pub block_state_write_radius: i32,
}

impl ChunkStep {
    pub fn accumulated_radius_of(&self, status: ChunkStatus) -> Option<usize> {
        if status == self.target_status {
            Some(0)
        } else {
            self.accumulated_dependencies.radius_of(status)
        }
    }

    /// Стадия, которую планировщик должен запросить у чанка на этом расстоянии.
    pub fn required_status_at_radius(&self, radius: usize) -> Option<ChunkStatus> {
        if radius == 0 && self.target_status == Empty {
            return Some(Empty);
        }
        self.accumulated_dependencies.at_radius(radius)
    }
}

/// Порядок и требования стадий для нового или уже сохранённого чанка.
#[derive(Debug)]
pub struct ChunkPyramid {
    steps: Vec<ChunkStep>,
}

pub static GENERATION_PYRAMID: LazyLock<ChunkPyramid> = LazyLock::new(|| {
    ChunkPyramid::build(&[
        (&[], -1),                                  // Empty
        (&[], -1),                                  // StructureStarts
        (&[(StructureStarts, 8)], -1),              // StructureReferences
        (&[(StructureStarts, 8)], -1),              // Biomes
        (&[(StructureStarts, 8), (Biomes, 1)], 0),  // Noise
        (&[(StructureStarts, 8), (Biomes, 1)], 0),  // Surface
        (&[(StructureStarts, 8)], 0),               // Carvers
        (&[(StructureStarts, 8), (Carvers, 1)], 1), // Features
        (&[], -1),                                  // InitializeLight
        (&[(InitializeLight, 1)], -1),              // Light
        (&[(Biomes, 1)], -1),                       // Spawn
        (&[], -1),                                  // Full
    ])
});

pub static LOADING_PYRAMID: LazyLock<ChunkPyramid> =
    LazyLock::new(|| ChunkPyramid::build(&[(&[], -1); ChunkStatus::VALUES.len()]));

impl ChunkPyramid {
    pub fn get_step_to(&self, status: ChunkStatus) -> &ChunkStep {
        &self.steps[status.ordinal()]
    }

    fn build(requirements: &[(&[(ChunkStatus, usize)], i32); ChunkStatus::VALUES.len()]) -> Self {
        let mut steps: Vec<ChunkStep> = Vec::with_capacity(requirements.len());
        for (status, &(requirements, write_radius)) in
            ChunkStatus::VALUES.into_iter().zip(requirements)
        {
            let mut direct = steps
                .last()
                .map_or_else(Vec::new, |parent| vec![parent.target_status]);
            for &(required, radius) in requirements {
                if direct.len() <= radius {
                    direct.resize(radius + 1, required);
                }
                for dependency in &mut direct[..=radius] {
                    if required.is_or_after(*dependency) {
                        *dependency = required;
                    }
                }
            }

            let direct_dependencies = ChunkDependencies { by_radius: direct };
            let mut accumulated = direct_dependencies.by_radius.clone();
            if let Some(parent) = steps.last() {
                let offset = direct_dependencies.radius_of(parent.target_status).unwrap();
                for (radius, &dependency) in parent
                    .accumulated_dependencies
                    .as_slice()
                    .iter()
                    .enumerate()
                {
                    let radius = offset + radius;
                    if let Some(required) = accumulated.get_mut(radius) {
                        if dependency.is_or_after(*required) {
                            *required = dependency;
                        }
                    } else {
                        accumulated.push(dependency);
                    }
                }
            }
            steps.push(ChunkStep {
                target_status: status,
                direct_dependencies,
                accumulated_dependencies: ChunkDependencies {
                    by_radius: accumulated,
                },
                block_state_write_radius: write_radius,
            });
        }
        Self { steps }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_generation_includes_transitive_dependencies() {
        let full = GENERATION_PYRAMID.get_step_to(Full);
        assert_eq!(full.direct_dependencies.as_slice(), &[Spawn]);
        assert_eq!(full.accumulated_radius_of(Biomes), Some(3));
        assert_eq!(full.accumulated_radius_of(StructureStarts), Some(11));
        assert_eq!(full.required_status_at_radius(1), Some(InitializeLight));
        assert_eq!(full.required_status_at_radius(2), Some(Carvers));
        assert_eq!(full.required_status_at_radius(12), None);
        assert_eq!(
            GENERATION_PYRAMID
                .get_step_to(Features)
                .block_state_write_radius,
            1
        );
    }

    #[test]
    fn loading_does_not_require_neighbours_for_starlight() {
        let light = LOADING_PYRAMID.get_step_to(Light);
        assert_eq!(light.direct_dependencies.as_slice(), &[InitializeLight]);
        assert_eq!(light.accumulated_radius_of(Empty), Some(0));
        assert_eq!(light.required_status_at_radius(1), None);
        assert_eq!(
            LOADING_PYRAMID
                .get_step_to(Empty)
                .required_status_at_radius(0),
            Some(Empty)
        );
    }
}
