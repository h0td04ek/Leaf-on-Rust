//! `net.minecraft.world.level.biome.Climate` — климатические параметры,
//! RTree-поиск по ParameterPoint, Sampler, SpawnFinder.

use super::super::levelgen::density_function::{DensityFunction, SinglePointContext};
use super::super::levelgen::density_functions::{self as df, SharedFunction};
use crate::core::block_pos::BlockPos;
use crate::core::quart_pos;
use crate::util::mth;
use std::cmp::Ordering;
use std::sync::Arc;

/// `PARAMETER_COUNT`.
pub const PARAMETER_COUNT: usize = 7;

/// `Climate.target(temperature, humidity, continentalness, erosion, depth, weirdness)`.
pub fn target(
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    depth: f32,
    weirdness: f32,
) -> TargetPoint {
    TargetPoint {
        temperature: quantize_coord(temperature),
        humidity: quantize_coord(humidity),
        continentalness: quantize_coord(continentalness),
        erosion: quantize_coord(erosion),
        depth: quantize_coord(depth),
        weirdness: quantize_coord(weirdness),
    }
}

/// `Climate.parameters` (все point).
#[allow(clippy::too_many_arguments)]
pub fn parameters_points(
    temperature: f32,
    humidity: f32,
    continentalness: f32,
    erosion: f32,
    depth: f32,
    weirdness: f32,
    offset: f32,
) -> ParameterPoint {
    ParameterPoint {
        temperature: Parameter::point(temperature),
        humidity: Parameter::point(humidity),
        continentalness: Parameter::point(continentalness),
        erosion: Parameter::point(erosion),
        depth: Parameter::point(depth),
        weirdness: Parameter::point(weirdness),
        offset: quantize_coord(offset),
    }
}

/// `Climate.parameters` (интервалы).
#[allow(clippy::too_many_arguments)]
pub fn parameters(
    temperature: Parameter,
    humidity: Parameter,
    continentalness: Parameter,
    erosion: Parameter,
    depth: Parameter,
    weirdness: Parameter,
    offset: f32,
) -> ParameterPoint {
    ParameterPoint { temperature, humidity, continentalness, erosion, depth, weirdness, offset: quantize_coord(offset) }
}

/// `Climate.quantizeCoord`.
pub fn quantize_coord(coord: f32) -> i64 {
    (coord * 10000.0) as i64
}

/// `Climate.unquantizeCoord`.
pub fn unquantize_coord(coord: i64) -> f32 {
    coord as f32 / 10000.0
}

/// `Climate.empty()` — пустой sampler.
pub fn empty_sampler() -> Sampler {
    let d = df::zero();
    Sampler {
        temperature: d.clone(),
        humidity: d.clone(),
        continentalness: d.clone(),
        erosion: d.clone(),
        depth: d.clone(),
        weirdness: d,
        spawn_target: Vec::new(),
    }
}

/// `Climate.findSpawnPosition(points, sampler)`.
pub fn find_spawn_position(points: &[ParameterPoint], sampler: &Sampler) -> BlockPos {
    SpawnFinder::new(points, sampler).result.location
}

/// `Climate.DistanceMetric<T>`.
pub trait DistanceMetric<T>: Send + Sync {
    /// `distance(node, searchedValues)`.
    fn distance(&self, node: &RTreeNode<T>, searched_values: &[i64; PARAMETER_COUNT]) -> i64;
}

/// `Climate.RTree.Node::distance` — метрика по умолчанию.
#[derive(Clone, Copy, Default)]
pub struct NodeDistance;

impl<T> DistanceMetric<T> for NodeDistance {
    fn distance(&self, node: &RTreeNode<T>, searched_values: &[i64; PARAMETER_COUNT]) -> i64 {
        node.distance(searched_values)
    }
}

/// `Climate.Parameter(min, max)` — квантованный интервал [-2.0, 2.0].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Parameter {
    /// `min`.
    pub min: i64,
    /// `max`.
    pub max: i64,
}

impl Parameter {
    /// `point(value)`.
    pub fn point(value: f32) -> Self {
        Self::span_floats(value, value)
    }
    /// `span(min, max)` floats.
    pub fn span_floats(min: f32, max: f32) -> Self {
        assert!(min <= max, "min > max: {} {}", min, max);
        Self { min: quantize_coord(min), max: quantize_coord(max) }
    }
    /// `span(min, max)` Parameters.
    pub fn span_params(min: Parameter, max: Parameter) -> Self {
        assert!(min.min <= max.max, "min > max: {:?} {:?}", min, max);
        Self { min: min.min, max: max.max }
    }
    /// `new Parameter(min, max)` raw.
    pub fn new(min: i64, max: i64) -> Self {
        Self { min, max }
    }

    /// `distance(pointValue)`.
    pub fn distance_to_value(&self, point_value: i64) -> i64 {
        let l = point_value - self.max;
        let l1 = self.min - point_value;
        if l > 0 { l } else { l1.max(0) }
    }

    /// `distance(parameter)`.
    pub fn distance_to_parameter(&self, parameter: Parameter) -> i64 {
        let l = parameter.min - self.max;
        let l1 = self.min - parameter.max;
        if l > 0 { l } else { l1.max(0) }
    }

    /// `span(param)` — расширение интервала.
    pub fn span(&self, param: Option<Parameter>) -> Parameter {
        match param {
            None => *self,
            Some(p) => Parameter { min: self.min.min(p.min), max: self.max.max(p.max) },
        }
    }
}

/// `Climate.ParameterPoint` — точка в 7-мерном климатическом пространстве.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParameterPoint {
    /// `temperature`.
    pub temperature: Parameter,
    /// `humidity`.
    pub humidity: Parameter,
    /// `continentalness`.
    pub continentalness: Parameter,
    /// `erosion`.
    pub erosion: Parameter,
    /// `depth`.
    pub depth: Parameter,
    /// `weirdness`.
    pub weirdness: Parameter,
    /// `offset`.
    pub offset: i64,
}

impl ParameterPoint {
    /// `fitness(targetPoint)`.
    pub fn fitness(&self, point: &TargetPoint) -> i64 {
        mth::square_i64(self.temperature.distance_to_value(point.temperature))
            + mth::square_i64(self.humidity.distance_to_value(point.humidity))
            + mth::square_i64(self.continentalness.distance_to_value(point.continentalness))
            + mth::square_i64(self.erosion.distance_to_value(point.erosion))
            + mth::square_i64(self.depth.distance_to_value(point.depth))
            + mth::square_i64(self.weirdness.distance_to_value(point.weirdness))
            + mth::square_i64(self.offset)
    }

    /// `parameterSpace()` — 7 параметров (offset как точка).
    pub fn parameter_space(&self) -> [Parameter; PARAMETER_COUNT] {
        [
            self.temperature,
            self.humidity,
            self.continentalness,
            self.erosion,
            self.depth,
            self.weirdness,
            Parameter { min: self.offset, max: self.offset },
        ]
    }
}

/// `Climate.TargetPoint`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TargetPoint {
    /// `temperature`.
    pub temperature: i64,
    /// `humidity`.
    pub humidity: i64,
    /// `continentalness`.
    pub continentalness: i64,
    /// `erosion`.
    pub erosion: i64,
    /// `depth`.
    pub depth: i64,
    /// `weirdness`.
    pub weirdness: i64,
}

impl TargetPoint {
    /// `toParameterArray`.
    pub fn to_parameter_array(&self) -> [i64; PARAMETER_COUNT] {
        [self.temperature, self.humidity, self.continentalness, self.erosion, self.depth, self.weirdness, 0]
    }
}

/// `Climate.ParameterList<T>` — список (ParameterPoint, T) с RTree-индексом.
pub struct ParameterList<T> {
    values: Vec<(ParameterPoint, T)>,
    index: RTree<T>,
}

impl<T: Clone + Send + Sync + 'static> ParameterList<T> {
    /// `new ParameterList(values)`.
    pub fn new(values: Vec<(ParameterPoint, T)>) -> Self {
        let index = RTree::create(&values);
        Self { values, index }
    }

    /// `values()`.
    pub fn values(&self) -> &[(ParameterPoint, T)] {
        &self.values
    }

    /// `findValue(targetPoint)`.
    pub fn find_value(&self, target_point: &TargetPoint) -> &T {
        self.find_value_index(target_point, &NodeDistance)
    }

    /// `findValueBruteForce` (для тестов).
    pub fn find_value_brute_force(&self, target_point: &TargetPoint) -> &T {
        let mut best = &self.values[0];
        let mut best_fitness = best.0.fitness(target_point);
        for pair in &self.values[1..] {
            let f = pair.0.fitness(target_point);
            if f < best_fitness {
                best_fitness = f;
                best = pair;
            }
        }
        &best.1
    }

    /// `findValueIndex(targetPoint, distanceMetric)`.
    pub fn find_value_index(&self, target_point: &TargetPoint, distance_metric: &dyn DistanceMetric<T>) -> &T {
        self.index.search(target_point, distance_metric)
    }
}

/// `Climate.RTree<T>` — поисковое дерево.
pub struct RTree<T> {
    root: RTreeNode<T>,
}

/// `Climate.RTree.Node<T>` — узел дерева.
pub enum RTreeNode<T> {
    /// `Leaf`.
    Leaf {
        /// `parameterSpace`.
        parameter_space: [Parameter; PARAMETER_COUNT],
        /// `value`.
        value: T,
    },
    /// `SubTree`.
    SubTree {
        /// `parameterSpace`.
        parameter_space: [Parameter; PARAMETER_COUNT],
        /// `children`.
        children: Vec<RTreeNode<T>>,
    },
}

/// `Climate.RTree.SubTree` — бакет узлов с агрегированным parameterSpace.
struct SubTreeBucket<T> {
    parameter_space: [Parameter; PARAMETER_COUNT],
    nodes: Vec<RTreeNode<T>>,
}

impl<T> RTreeNode<T> {
    fn parameter_space(&self) -> &[Parameter; PARAMETER_COUNT] {
        match self {
            RTreeNode::Leaf { parameter_space, .. } | RTreeNode::SubTree { parameter_space, .. } => parameter_space,
        }
    }

    /// `Node.distance(values)`.
    fn distance(&self, values: &[i64; PARAMETER_COUNT]) -> i64 {
        let mut l = 0i64;
        for i in 0..PARAMETER_COUNT {
            l += mth::square_i64(self.parameter_space()[i].distance_to_value(values[i]));
        }
        l
    }

    /// `Node.search(values, leaf, metric)` — возвращает лучший лист.
    fn search<'a>(
        &'a self,
        searched_values: &[i64; PARAMETER_COUNT],
        leaf: Option<&'a RTreeNode<T>>,
        metric: &dyn DistanceMetric<T>,
    ) -> Option<&'a RTreeNode<T>> {
        match self {
            RTreeNode::Leaf { .. } => Some(self),
            RTreeNode::SubTree { children, .. } => {
                let mut l = leaf.map(|lf| metric.distance(lf, searched_values)).unwrap_or(i64::MAX);
                let mut leaf1 = leaf;
                for node in children {
                    let l1 = metric.distance(node, searched_values);
                    if l > l1 {
                        let leaf2 = node.search(searched_values, leaf1, metric);
                        let l2 = leaf2.map(|n| metric.distance(n, searched_values)).unwrap_or(l1);
                        if l > l2 {
                            l = l2;
                            leaf1 = leaf2;
                        }
                    }
                }
                leaf1
            }
        }
    }
}

impl<T: Clone + Send + Sync + 'static> RTree<T> {
    /// `RTree.create(nodes)`.
    pub fn create(nodes: &[(ParameterPoint, T)]) -> Self {
        assert!(!nodes.is_empty(), "Need at least one value to build the search tree.");
        let mut leaves: Vec<RTreeNode<T>> = nodes
            .iter()
            .map(|(p, v)| RTreeNode::Leaf { parameter_space: p.parameter_space(), value: v.clone() })
            .collect();
        let root = Self::build(PARAMETER_COUNT, std::mem::take(&mut leaves));
        Self { root }
    }

    /// `build(paramSpaceSize, children)`.
    fn build(param_space_size: usize, children: Vec<RTreeNode<T>>) -> RTreeNode<T> {
        assert!(!children.is_empty(), "Need at least one child to build a node");
        if children.len() == 1 {
            return children.into_iter().next().unwrap();
        }
        if children.len() <= 6 {
            let mut children = children;
            // Java: sort по сумме |(min+max)/2| по всем измерениям.
            children.sort_by_key(|child| {
                let mut l2 = 0i64;
                for parameter in child.parameter_space().iter() {
                    l2 += ((parameter.min + parameter.max) / 2).abs();
                }
                l2
            });
            let parameter_space = Self::build_parameter_space(children.iter());
            return RTreeNode::SubTree { parameter_space, children };
        }

        let mut children = children;
        let mut best: Option<(i64, usize)> = None;
        for i1 in 0..param_space_size {
            Self::sort_nodes(&mut children, param_space_size, i1, false);
            let ranges = Self::bucket_ranges(children.len());
            let mut l1 = 0i64;
            for &(s, e) in &ranges {
                l1 += Self::cost(&Self::build_parameter_space(children[s..e].iter()));
            }
            if best.as_ref().map_or(true, |(bl, _)| l1 < *bl) {
                best = Some((l1, i1));
            }
        }
        let (_, best_dim) = best.unwrap();

        // Java: bucketize по порядку sort(dim,false), затем sort(list,dim,true)
        // сортирует сами bucket'и. Воспроизводим: пересортировываем children
        // в том же порядке и режем на bucket'и.
        Self::sort_nodes(&mut children, param_space_size, best_dim, false);
        let ranges = Self::bucket_ranges(children.len());
        let mut buckets: Vec<SubTreeBucket<T>> = Vec::with_capacity(ranges.len());
        let mut rest = children;
        let mut offset = 0usize;
        for (s, e) in ranges {
            let take = e - s;
            debug_assert_eq!(s, offset);
            let tail = rest.split_off(take);
            let nodes = std::mem::replace(&mut rest, tail);
            offset = e;
            let parameter_space = Self::build_parameter_space(nodes.iter());
            buckets.push(SubTreeBucket { parameter_space, nodes });
        }

        // sort(list, paramSpaceSize, best_dim, true) — сортировка bucket'ов
        // по их агрегированному parameterSpace (цепочка компараторов).
        buckets.sort_by(|a, b| {
            for k in 0..param_space_size {
                let idx = (best_dim + k) % param_space_size;
                let ord = Self::node_cmp(a.parameter_space[idx], b.parameter_space[idx], true);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            Ordering::Equal
        });

        let built: Vec<RTreeNode<T>> = buckets
            .into_iter()
            .map(|bucket| Self::build(param_space_size, bucket.nodes))
            .collect();
        let parameter_space = Self::build_parameter_space(built.iter());
        RTreeNode::SubTree { parameter_space, children: built }
    }

    /// `sort(children, paramSpaceSize, size, absolute)`.
    fn sort_nodes(children: &mut [RTreeNode<T>], param_space_size: usize, size: usize, absolute: bool) {
        children.sort_by(|a, b| {
            for k in 0..param_space_size {
                let idx = (size + k) % param_space_size;
                let ord = Self::node_cmp(a.parameter_space()[idx], b.parameter_space()[idx], absolute);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            Ordering::Equal
        });
    }

    fn node_cmp(pa: Parameter, pb: Parameter, absolute: bool) -> Ordering {
        let la = (pa.min + pa.max) / 2;
        let lb = (pb.min + pb.max) / 2;
        let (va, vb) = if absolute { (la.abs(), lb.abs()) } else { (la, lb) };
        va.cmp(&vb)
    }

    /// `bucketize` — диапазоны bucket'ов по размеру `i`.
    fn bucket_ranges(len: usize) -> Vec<(usize, usize)> {
        let i = 6f64.powf((((len as f64) - 0.01).ln() / 6f64.ln()).floor());
        let cap = i as usize;
        let mut ranges = Vec::new();
        let mut s = 0;
        while s < len {
            let e = (s + cap).min(len);
            ranges.push((s, e));
            s = e;
        }
        ranges
    }

    /// `cost(parameters)`.
    fn cost(parameters: &[Parameter; PARAMETER_COUNT]) -> i64 {
        let mut l = 0i64;
        for p in parameters {
            l += (p.max - p.min).abs();
        }
        l
    }

    /// `buildParameterSpace(children)`.
    fn build_parameter_space<'a, I>(children: I) -> [Parameter; PARAMETER_COUNT]
    where
        I: IntoIterator<Item = &'a RTreeNode<T>>,
    {
        let mut list: [Option<Parameter>; PARAMETER_COUNT] = [None; PARAMETER_COUNT];
        for node in children {
            for i2 in 0..PARAMETER_COUNT {
                list[i2] = Some(node.parameter_space()[i2].span(list[i2]));
            }
        }
        list.map(|p| p.unwrap())
    }

    /// `search(targetPoint, distanceMetric)`.
    pub fn search(&self, target_point: &TargetPoint, distance_metric: &dyn DistanceMetric<T>) -> &T {
        let longs = target_point.to_parameter_array();
        let leaf = self.root.search(&longs, None, distance_metric).expect("RTree.search не нашёл лист");
        match leaf {
            RTreeNode::Leaf { value, .. } => value,
            RTreeNode::SubTree { .. } => unreachable!("search возвращает лист"),
        }
    }
}

/// `Climate.Sampler` — сэмплер климатических параметров из DensityFunction.
pub struct Sampler {
    /// `temperature`.
    pub temperature: SharedFunction,
    /// `humidity`.
    pub humidity: SharedFunction,
    /// `continentalness`.
    pub continentalness: SharedFunction,
    /// `erosion`.
    pub erosion: SharedFunction,
    /// `depth`.
    pub depth: SharedFunction,
    /// `weirdness`.
    pub weirdness: SharedFunction,
    /// `spawnTarget`.
    pub spawn_target: Vec<ParameterPoint>,
}

impl Sampler {
    /// `sample(x, y, z)` — x/y/z в quart-координатах.
    pub fn sample(&self, x: i32, y: i32, z: i32) -> TargetPoint {
        let block_pos_x = quart_pos::to_block(x);
        let block_pos_y = quart_pos::to_block(y);
        let block_pos_z = quart_pos::to_block(z);
        let ctx = SinglePointContext::new(block_pos_x, block_pos_y, block_pos_z);
        target(
            self.temperature.compute(&ctx) as f32,
            self.humidity.compute(&ctx) as f32,
            self.continentalness.compute(&ctx) as f32,
            self.erosion.compute(&ctx) as f32,
            self.depth.compute(&ctx) as f32,
            self.weirdness.compute(&ctx) as f32,
        )
    }

    /// `findSpawnPosition`.
    pub fn find_spawn_position(&self) -> BlockPos {
        if self.spawn_target.is_empty() {
            BlockPos::new(0, 0, 0)
        } else {
            find_spawn_position(&self.spawn_target, self)
        }
    }
}

/// `Climate.SpawnFinder`.
pub struct SpawnFinder {
    /// `result`.
    pub result: SpawnResult,
}

/// `SpawnFinder.Result(location, fitness)`.
#[derive(Clone, Copy, Debug)]
pub struct SpawnResult {
    /// `location`.
    pub location: BlockPos,
    /// `fitness`.
    pub fitness: i64,
}

impl SpawnFinder {
    const MAX_RADIUS: f32 = 2048.0;

    /// `new SpawnFinder(points, sampler)`.
    pub fn new(points: &[ParameterPoint], sampler: &Sampler) -> Self {
        let mut result = Self::get_spawn_position_and_fitness(points, sampler, 0, 0);
        Self::radial_search(points, sampler, &mut result, Self::MAX_RADIUS, 512.0);
        Self::radial_search(points, sampler, &mut result, 512.0, 32.0);
        Self { result }
    }

    /// `radialSearch`.
    fn radial_search(points: &[ParameterPoint], sampler: &Sampler, result: &mut SpawnResult, max: f32, min: f32) {
        let mut f = 0.0f32;
        let mut f1 = min;
        let block_pos = result.location;
        while f1 <= max {
            let i = block_pos.x + (f.sin() * f1) as i32;
            let i1 = block_pos.z + (f.cos() * f1) as i32;
            let spawn = Self::get_spawn_position_and_fitness(points, sampler, i, i1);
            if spawn.fitness < result.fitness {
                *result = spawn;
            }
            f += min / f1;
            if f > std::f32::consts::TAU {
                f = 0.0;
                f1 += min;
            }
        }
    }

    /// `getSpawnPositionAndFitness`.
    fn get_spawn_position_and_fitness(points: &[ParameterPoint], sampler: &Sampler, x: i32, z: i32) -> SpawnResult {
        let target_point = sampler.sample(quart_pos::from_block(x), 0, quart_pos::from_block(z));
        let target_point1 = TargetPoint {
            temperature: target_point.temperature,
            humidity: target_point.humidity,
            continentalness: target_point.continentalness,
            erosion: target_point.erosion,
            depth: 0,
            weirdness: target_point.weirdness,
        };
        let mut l = i64::MAX;
        for parameter_point in points {
            l = l.min(parameter_point.fitness(&target_point1));
        }
        let l1 = mth::square_i64(x as i64) + mth::square_i64(z as i64);
        let l2 = l * mth::square_i64(2048) + l1;
        SpawnResult { location: BlockPos::new(x, 0, z), fitness: l2 }
    }
}
