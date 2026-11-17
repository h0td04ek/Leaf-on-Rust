//! Перенос `org.apache.commons.lang3.math.Fraction` —
//! дробь numerator/denominator (int, всегда нормализована).
//! Используется `BundleContents` для веса содержимого.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// `Fraction` — несократимая дробь со знаком в числителе.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fraction {
    /// `numerator`.
    pub numerator: i64,
    /// `denominator` (>0).
    pub denominator: i64,
}

impl Fraction {
    /// `ZERO`.
    pub const ZERO: Fraction = Fraction { numerator: 0, denominator: 1 };
    /// `ONE`.
    pub const ONE: Fraction = Fraction { numerator: 1, denominator: 1 };
    /// `MINUS_ONE`.
    pub const MINUS_ONE: Fraction = Fraction { numerator: -1, denominator: 1 };

    /// `getFraction(numerator, denominator)` — нормализует.
    /// Как Apache: `denominator == 0` → ArithmeticException;
    /// знак в числителе, сокращение по gcd.
    pub fn new(numerator: i64, denominator: i64) -> Self {
        assert!(denominator != 0, "The denominator must not be zero");
        if (numerator == 0) {
            return Fraction::ZERO;
        }
        let mut num = numerator;
        let mut den = denominator;
        if (den < 0) {
            num = -num;
            den = -den;
        }
        let g = gcd(num, den);
        Fraction { numerator: num / g, denominator: den / g }
    }

    /// `getReducedFraction(numerator, denominator)`.
    pub fn get_reduced(numerator: i64, denominator: i64) -> Self {
        Self::new(numerator, denominator)
    }

    /// `getNumerator`.
    pub fn get_numerator(&self) -> i64 {
        self.numerator
    }

    /// `getDenominator`.
    pub fn get_denominator(&self) -> i64 {
        self.denominator
    }

    /// `add(f)` / `add(n, d)`.
    pub fn add(self, other: Fraction) -> Fraction {
        self + other
    }

    /// `subtract`.
    pub fn subtract(self, other: Fraction) -> Fraction {
        self - other
    }

    /// `multiplyBy` / `multiply`.
    pub fn multiply_by(self, other: Fraction) -> Fraction {
        self * other
    }

    /// `multiply(int)`.
    pub fn multiply(self, n: i64) -> Fraction {
        self * Fraction::new(n, 1)
    }

    /// `divide`.
    pub fn divide(self, other: Fraction) -> Fraction {
        Fraction::new(self.numerator * other.denominator, self.denominator * other.numerator)
    }

    /// `negate`.
    pub fn negate(self) -> Fraction {
        Fraction { numerator: -self.numerator, denominator: self.denominator }
    }

    /// `invert`.
    pub fn invert(self) -> Fraction {
        Fraction::new(self.denominator, self.numerator)
    }

    /// `pow`.
    pub fn pow(self, power: u32) -> Fraction {
        let mut result = Fraction::ONE;
        for _ in 0..power {
            result = result * self;
        }
        result
    }

    /// `abs`.
    pub fn abs(self) -> Fraction {
        Fraction { numerator: self.numerator.abs(), denominator: self.denominator }
    }

    /// `signum`.
    pub fn signum(&self) -> i64 {
        self.numerator.signum()
    }

    /// `compareTo`.
    pub fn compare_to(&self, other: &Fraction) -> Ordering {
        let left = self.numerator as i128 * other.denominator as i128;
        let right = other.numerator as i128 * self.denominator as i128;
        left.cmp(&right)
    }

    /// `reduce()` — повторное сокращение (уже нормализована).
    pub fn reduce(self) -> Fraction {
        self
    }

    /// `getPercentage` / `doubleValue` — double-значение.
    pub fn double_value(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    /// `floatValue`.
    pub fn float_value(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }
}

impl Default for Fraction {
    fn default() -> Self {
        Fraction::ZERO
    }
}

impl Add for Fraction {
    type Output = Fraction;
    fn add(self, other: Fraction) -> Fraction {
        // a/b + c/d = (ad + bc)/bd — Apache делает через gcd-оптимизацию;
        // результат тот же после нормализации.
        Fraction::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}

impl Add<i64> for Fraction {
    type Output = Fraction;
    fn add(self, n: i64) -> Fraction {
        self + Fraction::new(n, 1)
    }
}

impl Sub for Fraction {
    type Output = Fraction;
    fn sub(self, other: Fraction) -> Fraction {
        Fraction::new(
            self.numerator * other.denominator - other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }
}

impl Sub<i64> for Fraction {
    type Output = Fraction;
    fn sub(self, n: i64) -> Fraction {
        self - Fraction::new(n, 1)
    }
}

impl Mul for Fraction {
    type Output = Fraction;
    fn mul(self, other: Fraction) -> Fraction {
        Fraction::new(self.numerator * other.numerator, self.denominator * other.denominator)
    }
}

impl Mul<i64> for Fraction {
    type Output = Fraction;
    fn mul(self, n: i64) -> Fraction {
        self * Fraction::new(n, 1)
    }
}

impl std::ops::Div for Fraction {
    type Output = Fraction;
    fn div(self, other: Fraction) -> Fraction {
        self.divide(other)
    }
}

impl Neg for Fraction {
    type Output = Fraction;
    fn neg(self) -> Fraction {
        self.negate()
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.compare_to(other))
    }
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_to(other)
    }
}

impl fmt::Display for Fraction {
    /// `toString` — "n" если d==1, иначе "n/d".
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Fraction;

    #[test]
    fn normalize() {
        let f = Fraction::new(2, 4);
        assert_eq!(f.numerator, 1);
        assert_eq!(f.denominator, 2);
        let f = Fraction::new(-2, -4);
        assert_eq!(f.numerator, 1);
        assert_eq!(f.denominator, 2);
        assert_eq!(Fraction::new(0, 5), Fraction::ZERO);
    }

    #[test]
    fn arithmetic() {
        let a = Fraction::new(1, 2);
        let b = Fraction::new(1, 3);
        assert_eq!(a + b, Fraction::new(5, 6));
        assert_eq!(a - b, Fraction::new(1, 6));
        assert_eq!(a * b, Fraction::new(1, 6));
        assert_eq!(a / b, Fraction::new(3, 2));
        assert_eq!(a.multiply_by(Fraction::new(4, 1)), Fraction::new(2, 1));
        assert_eq!(a.negate(), Fraction::new(-1, 2));
    }

    #[test]
    fn compare_and_string() {
        assert!(Fraction::new(1, 2) < Fraction::new(2, 3));
        assert_eq!(Fraction::new(1, 1).to_string(), "1");
        assert_eq!(Fraction::new(1, 16).to_string(), "1/16");
    }
}
