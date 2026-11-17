//! Минимальный `com.mojang.datafixers.util.Either`.

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Either<L, R> {
    pub fn left(value: L) -> Self {
        Self::Left(value)
    }

    pub fn right(value: R) -> Self {
        Self::Right(value)
    }

    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left(_))
    }

    pub fn is_right(&self) -> bool {
        matches!(self, Self::Right(_))
    }

    pub fn map_left<T>(self, f: impl FnOnce(L) -> T) -> Either<T, R> {
        match self {
            Self::Left(l) => Either::Left(f(l)),
            Self::Right(r) => Either::Right(r),
        }
    }

    pub fn map_right<T>(self, f: impl FnOnce(R) -> T) -> Either<L, T> {
        match self {
            Self::Left(l) => Either::Left(l),
            Self::Right(r) => Either::Right(f(r)),
        }
    }
}
