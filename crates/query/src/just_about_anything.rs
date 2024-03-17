/// "Lightweight" [`std::any::Any`] that does not provide any type metadata; it's just [`Drop`]pable.
pub trait JustAboutAnything<'e>: 'e {}
impl<'e, T: 'e> JustAboutAnything<'e> for T {}

/// # Safety
///
/// The user must be sure that the actual data referenced by `anything` is of type `T`.
pub unsafe fn transmute<'a, T>(anything: &'a dyn JustAboutAnything<'_>) -> &'a T {
    &*(anything as *const dyn JustAboutAnything as *const () as *const T)
}
