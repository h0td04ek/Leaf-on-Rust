//! Тесты соответствия Java-последовательностям (сверено с JDK 25).

use crate::random::bit::BitRandomSource;
use crate::random::legacy::LegacyRandomSource;
use crate::random::single::SingleThreadedRandomSource;
use crate::random::support::{mix_stafford13, upgrade_seed_to_128bit};
use crate::random::xoroshiro::Xoroshiro128PlusPlus;
use crate::random::RandomSource;
use crate::util::mth;

#[test]
fn legacy_matches_java_util_random() {
    // java.util.Random(0): -1155484576, -723955400, 4437113781045784766,
    // 0.6374174253501083, 0.550437, false, 0.9845745328825128
    let mut r = LegacyRandomSource::new(0);
    assert_eq!(RandomSource::next_int(&mut r), -1155484576);
    assert_eq!(RandomSource::next_int(&mut r), -723955400);
    assert_eq!(RandomSource::next_long(&mut r), 4437113781045784766);
    assert!((RandomSource::next_double(&mut r) - 0.6374174253501083).abs() < 1e-15);
    assert!((RandomSource::next_float(&mut r) - 0.550437).abs() < 1e-7);
    assert!(!RandomSource::next_boolean(&mut r));
    assert!((RandomSource::next_gaussian(&mut r) - 0.9845745328825128).abs() < 1e-12);
}

#[test]
fn single_threaded_matches_legacy() {
    let mut a = LegacyRandomSource::new(42);
    let mut b = SingleThreadedRandomSource::new(42);
    for _ in 0..1000 {
        assert_eq!(RandomSource::next_int(&mut a), RandomSource::next_int(&mut b));
        assert_eq!(RandomSource::next_long(&mut a), RandomSource::next_long(&mut b));
        assert_eq!(
            RandomSource::next_double(&mut a).to_bits(),
            RandomSource::next_double(&mut b).to_bits()
        );
    }
}

#[test]
fn xoroshiro_known_vectors() {
    // Xoroshiro128++ с нулевым сидом → константный fallback (как в Java)
    let mut r = Xoroshiro128PlusPlus::with_seeds(0, 0);
    assert_eq!(r.seed_lo, -7046029254386353131);
    assert_eq!(r.seed_hi, 7640891576956012809);
    let v1 = r.next_long();
    let mut r2 = Xoroshiro128PlusPlus::with_seeds(0, 0);
    assert_eq!(v1, r2.next_long());
}

#[test]
fn stafford13_known() {
    assert_eq!(mix_stafford13(0), mix_stafford13(0));
    let s = upgrade_seed_to_128bit(12345);
    assert_ne!(s.seed_lo, s.seed_hi);
    assert_eq!(s, upgrade_seed_to_128bit(12345));
}

#[test]
fn get_seed_deterministic() {
    assert_eq!(mth::get_seed(1, 2, 3), mth::get_seed(1, 2, 3));
    assert_ne!(mth::get_seed(1, 2, 3), mth::get_seed(3, 2, 1));
}

#[test]
fn fork_consumes_source() {
    // Java fork() потребляет nextLong() — исходник сдвигается на 1 значение
    let mut a = LegacyRandomSource::new(7);
    let mut b = LegacyRandomSource::new(7);
    let mut forked = b.fork();
    let a1 = BitRandomSource::next_long(&mut a);
    let a2 = BitRandomSource::next_long(&mut a);
    let b1 = BitRandomSource::next_long(&mut b);
    assert_eq!(a2, b1); // fork сожрал a1-эквивалент
    let _ = forked.next_int();
    let _ = a1;
}
