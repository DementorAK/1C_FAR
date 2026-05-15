/// Cross-platform wide string utilities for Far Plugin API.
///
/// FAR 3 / Windows: wchar_t = u16 (UTF-16)
/// far2l / Linux:   wchar_t = u32 (UTF-32)
///
/// Use `to_wide` / `from_wide_ptr` in all API boundary code instead of
/// calling `encode_utf16()` directly, to ensure the correct encoding is
/// used for the current build target.

// ── FAR 3 (far3 feature): wchar_t = u16 (UTF-16) ───────────────────────────

#[cfg(feature = "far3")]
/// Convert a Rust &str to a NUL-terminated Vec<u16> (UTF-16).
pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0u16)).collect()
}

#[cfg(feature = "far3")]
/// Read a NUL-terminated *const u16 wide string into a Rust String.
///
/// # Safety
/// `ptr` must be a valid, NUL-terminated UTF-16 pointer.
pub unsafe fn from_wide_ptr(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0isize;
    while *ptr.offset(len) != 0 {
        len += 1;
    }
    String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len as usize))
}

#[cfg(feature = "far3")]
/// Convert a Rust &str to a NUL-terminated Vec<u32>.
/// On far3 this is unused at the API level, but provided for symmetry.
#[allow(dead_code)]
pub fn to_wide32(s: &str) -> Vec<u32> {
    s.chars().map(|c| c as u32).chain(std::iter::once(0u32)).collect()
}

// ── far2 (far2 feature): wchar_t = u32 (UTF-32) ─────────────────────────────

#[cfg(feature = "far2")]
/// Convert a Rust &str to a NUL-terminated Vec<u32> (UTF-32).
pub fn to_wide(s: &str) -> Vec<u32> {
    s.chars().map(|c| c as u32).chain(std::iter::once(0u32)).collect()
}

#[cfg(feature = "far2")]
/// Read a NUL-terminated *const u32 wide string into a Rust String.
///
/// # Safety
/// `ptr` must be a valid, NUL-terminated UTF-32 pointer.
pub unsafe fn from_wide_ptr(ptr: *const u32) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0isize;
    while *ptr.offset(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr, len as usize);
    slice.iter()
        .filter_map(|&c| char::from_u32(c))
        .collect()
}

#[cfg(feature = "far2")]
/// Convert a Rust &str to a NUL-terminated Vec<u16>.
/// On far2 this is unused at the API level, but provided for symmetry.
#[allow(dead_code)]
pub fn to_wide16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0u16)).collect()
}
