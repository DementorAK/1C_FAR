//! `FarHost` — abstraction over Far Manager API version differences.
//!
//! This trait defines the conceptual interface for interacting with different FAR APIs.
//! In the current Static Multi-Feature architecture (V3), the concrete
//! implementations are statically linked at compile time based on the active feature:
//! `far3` (FAR Manager 3, Windows), `far2l` (far2l, Linux/macOS), or `far2m` (far2m, Linux/macOS/BSD).
//!
//! The trait serves as a design abstraction and documentation for the
//! expected behavior of the platform-specific API layers.

/// Describes operations that differ between FAR 3 and FAR 2 Plugin APIs.
#[allow(dead_code)]
pub trait FarHost {
    // ── String types ────────────────────────────────────────────────────────

    /// The wide-character type used by this FAR version.
    /// FAR 3 (Windows): `u16`   (UTF-16)
    /// far2l  (Linux):  `u32`   (UTF-32)
    type WChar: Copy + Default + PartialEq + 'static;

    // ── String conversions ───────────────────────────────────────────────────

    /// Convert a Rust `&str` to a NUL-terminated wide string.
    fn to_wide(s: &str) -> Vec<Self::WChar>;

    /// Convert a NUL-terminated wide pointer back to a Rust `String`.
    ///
    /// # Safety
    /// `ptr` must be valid and NUL-terminated.
    unsafe fn from_wide_ptr(ptr: *const Self::WChar) -> String;

    // ── UI helpers ───────────────────────────────────────────────────────────

    /// Display a blocking message dialog.
    /// Returns the index of the button the user pressed (0-based).
    fn message(title: &str, lines: &[&str], buttons: &[&str]) -> isize;

    /// Update the taskbar / dialog progress indicator.
    fn show_progress(title: &str, message: &str, current: usize, total: usize);

    /// Reset progress indicator after a long operation.
    fn finish_progress();

    // ── Panel helpers ────────────────────────────────────────────────────────

    /// Return the full path of the currently highlighted item on the active
    /// file panel, or `None` if the panel is not a file panel / item is "..".
    fn get_current_panel_path() -> Option<String>;
}
