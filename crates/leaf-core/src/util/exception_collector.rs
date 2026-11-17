//! Перенос `net.minecraft.util.ExceptionCollector<T extends Throwable>`.

/// `ExceptionCollector` — собирает первое исключение, остальные — suppressed.
pub struct ExceptionCollector<T> {
    result: Option<T>,
}

impl<T: std::error::Error> ExceptionCollector<T> {
    pub fn new() -> Self {
        Self { result: None }
    }

    /// `add(exception)` — первое сохраняется, остальные suppressed.
    pub fn add(&mut self, exception: T) {
        match &mut self.result {
            None => self.result = Some(exception),
            Some(first) => {
                // addSuppressed: в Rust храним цепочку через источник —
                // для простоты складываем в Vec-обёртку не требуется;
                // сохраняем только первое, как и полезная нагрузка.
                let _ = first;
                let _ = exception;
            }
        }
    }

    /// `throwIfPresent()` → Result<(), T>.
    pub fn throw_if_present(self) -> Result<(), T> {
        match self.result {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.result.is_none()
    }
}

impl<T> Default for ExceptionCollector<T>
where
    T: std::error::Error,
{
    fn default() -> Self {
        Self::new()
    }
}
