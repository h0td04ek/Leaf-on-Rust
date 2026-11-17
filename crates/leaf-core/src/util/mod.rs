//! `util` — математические и прочие хелперы ядра.

pub mod abortable_iteration_consumer;
pub mod argb;
pub mod array_list_deque;
pub mod bit_storage;
pub mod bounded_float_function;
pub mod brightness;
pub mod by_id_map;
pub mod class_instance_multi_map;
pub mod color_rgba;
pub mod common_colors;
pub mod cubic_spline;
pub mod csv_output;
pub mod delegate_data_output;
pub mod dependency_sorter;
pub mod debug;
pub mod debugchart;
pub mod directory_lock;
pub mod ease;
pub mod easing_type;
pub mod either;
pub mod exception_collector;
pub mod fast_buffered_input_stream;
pub mod file_util;
pub mod file_zipper;
pub mod graph;
pub mod identifier_pattern;
pub mod inclusive_range;
pub mod keyframe;
pub mod keyframe_track;
pub mod keyframe_track_sampler;
pub mod lerp_function;
pub mod linear_congruential_generator;
pub mod memory_reserve;
pub mod month_day;
pub mod mth;
pub mod png_info;
pub mod segmented_angle_precision;
pub mod single_key_cache;
pub mod smooth_double;
pub mod signature_validator;
pub mod signer;
pub mod sorted_array_set;
pub mod special_dates;
pub mod static_cache_2d;
pub mod string_representable;
pub mod string_util;
pub mod tick_throttler;
pub mod time_source;
pub mod time_util;
pub mod to_float_function;
pub mod tri_state;
pub mod tuple;
pub mod unit;
pub mod uuid;
pub mod valueproviders;

pub mod random;

use crate::random::RandomSource;

/// `Util.shuffle(List, RandomSource)` — Fisher-Yates внутри списка.
pub fn shuffle<T>(list: &mut [T], random: &mut dyn RandomSource) {
    let size = list.len();
    for i in (2..=size).rev() {
        let random_int = random.next_int_bound(i as i32) as usize;
        list.swap(i - 1, random_int);
    }
}

/// `Util.getRandom(T[], RandomSource)`
pub fn get_random<T: Copy>(selections: &[T], random: &mut dyn RandomSource) -> T {
    selections[random.next_int_bound(selections.len() as i32) as usize]
}

/// `Util.getRandom(List, RandomSource)`
pub fn get_random_list<T: Clone>(selections: &[T], random: &mut dyn RandomSource) -> T {
    selections[random.next_int_bound(selections.len() as i32) as usize].clone()
}

/// `Util.shuffledCopy(T[], RandomSource)`
pub fn shuffled_copy<T: Clone>(array: &[T], random: &mut dyn RandomSource) -> Vec<T> {
    let mut list = array.to_vec();
    shuffle(&mut list, random);
    list
}

/// `Util.fixedSize` — берёт ровно `size` элементов или ошибку.
pub fn fixed_size<'a, T>(list: &'a [T], size: usize) -> Result<&'a [T], String> {
    if list.len() == size {
        Ok(list)
    } else {
        Err(format!("Expected list of size {}, but was {}", size, list.len()))
    }
}

/// `Util.growByHalf(value, minValue)`.
pub fn grow_by_half(value: i32, min_value: i32) -> i32 {
    ((value as i64 + (value >> 1) as i64).min(2147483639)).max(min_value as i64) as i32
}

/// `Util.make(Supplier)` / `make(object, consumer)`.
pub fn make<T>(supplier: impl FnOnce() -> T) -> T {
    supplier()
}

pub fn make_with<T>(mut object: T, consumer: impl FnOnce(&mut T)) -> T {
    consumer(&mut object);
    object
}

/// `Util.createIndexLookup(List)` — ToIntFunction: индекс элемента или -1.
pub fn create_index_lookup<T: PartialEq>(list: &[T]) -> impl Fn(&T) -> i32 + '_ {
    move |value: &T| {
        list.iter()
            .position(|x| x == value)
            .map(|i| i as i32)
            .unwrap_or(-1)
    }
}
