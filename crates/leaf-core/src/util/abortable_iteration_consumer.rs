//! Перенос `net.minecraft.util.AbortableIterationConsumer<T>`.

/// `Continuation` — продолжать или прервать итерацию.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Continuation {
    CONTINUE,
    ABORT,
}

impl Continuation {
    pub fn should_abort(self) -> bool {
        self == Self::ABORT
    }
}

/// `AbortableIterationConsumer<T>` — consumer, который может прервать обход.
pub trait AbortableIterationConsumer<T> {
    fn accept(&mut self, value: T) -> Continuation;
}

impl<T, F: FnMut(T) -> Continuation> AbortableIterationConsumer<T> for F {
    fn accept(&mut self, value: T) -> Continuation {
        self(value)
    }
}

/// `forConsumer(Consumer)` — обёртка обычного consumer.
pub fn for_consumer<T, F: FnMut(T)>(mut consumer: F) -> impl AbortableIterationConsumer<T> {
    move |value: T| {
        consumer(value);
        Continuation::CONTINUE
    }
}
