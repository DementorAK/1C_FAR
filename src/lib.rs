pub mod base;
pub mod far;
pub mod v8;

// ── Platform guard ────────────────────────────────────────────────────────────
//
// Feature `far2` targets far2l / far2m (Linux / macOS).
// It relies on WinPort symbols (RegCreateKeyExW, etc.) that are only available
// in the far2l runtime.  Building with `far2` on Windows is not supported.
#[cfg(all(feature = "far2", target_os = "windows"))]
compile_error!(
    "Feature `far2` is not supported on Windows. \
     Use `cargo build --features far2 --no-default-features --target x86_64-unknown-linux-gnu` \
     for cross-compilation to Linux (far2l / far2m)."
);

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
