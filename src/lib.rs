pub mod base;
pub mod far;
pub mod v8;

// ── Conditional export of Far Plugin API entry points ─────────────────────────
//
// Exactly ONE feature must be active at build time:
//   --features far3  (default)  → FAR Manager 3, Windows
//   --features far2  →  far2l / far2m, Linux/macOS
//
// The `pub use` below pulls all #[no_mangle] functions from the selected
// submodule into the crate root so they appear in the cdylib export table.

#[cfg(feature = "far3")]
pub use far::far3::exports::*;

#[cfg(feature = "far2")]
pub use far::far2::exports::*;
