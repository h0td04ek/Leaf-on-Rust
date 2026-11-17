//! Перенос `net.minecraft.util.Tuple<A, B>`.

/// `Tuple(a, b)` — простая пара значений.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Tuple<A, B> {
    pub a: A,
    pub b: B,
}

impl<A, B> Tuple<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }

    pub fn get_a(&self) -> &A {
        &self.a
    }

    pub fn set_a(&mut self, a: A) {
        self.a = a;
    }

    pub fn get_b(&self) -> &B {
        &self.b
    }

    pub fn set_b(&mut self, b: B) {
        self.b = b;
    }
}
