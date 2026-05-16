//! `FarHost` — abstraction over Far Manager API version differences.
//!
//! This trait is **not yet fully wired up** — it documents the intended
//! interface for the future dual-API implementation.  The concrete
//! implementations will live in `far::far3` (FAR 3, Windows) and
//! `far::far2` (far2l/far2m, Linux).
//!
//! For now (Phase 4A) the trait exists as a design artifact; the actual
//! call-sites in `panels.rs` / `ui.rs` still go through the concrete
//! API layer directly.  Phase 4B will complete the wiring.

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
