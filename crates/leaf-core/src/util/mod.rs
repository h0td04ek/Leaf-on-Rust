//! `util` — математические и прочие хелперы ядра.

pub mod argb;
pub mod mth;

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

/// `Util.fixedSize` — берёт ровно `size` элементов или ошибку.
pub fn fixed_size<'a, T>(list: &'a [T], size: usize) -> Result<&'a [T], String> {
    if list.len() == size {
        Ok(list)
    } else {
        Err(format!("Expected list of size {}, but was {}", size, list.len()))
    }
}
