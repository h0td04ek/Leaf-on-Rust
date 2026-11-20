//! `ca.spottedleaf.moonrise.patches.starlight.blockstate`.

/// `StarlightAbstractBlockState` — помечает blockstate, который условно
/// является полным непрозрачным для StarLight.
pub trait StarlightAbstractBlockState {
    /// `starlight$isConditionallyFullOpaque`.
    fn starlight_is_conditionally_full_opaque(&self) -> bool;
}
