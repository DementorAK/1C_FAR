//! Runtime API wrapper for far2l / far2m compatibility.
//!
//! Instead of relying on a single `PluginStartupInfo` struct layout,
//! `FarApi` extracts function pointers from the raw bytes passed by
//! the host, using byte-offset tables keyed by the detected variant.

#![allow(non_snake_case)]
#![allow(dead_code)]

use crate::far::far2::api::*;
use log::info;

// ── Host variant detection ──────────────────────────────────────────────

/// Detected host variant based on `StructSize`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FarHostVariant {
    /// far2l 2.6.x (Ubuntu repo) — StructSize = 260
    Far2l26,
    /// far2l 2.8.x (GitHub) — StructSize = 276
    Far2l28,
    /// far2m (GitHub, shmuz) — StructSize = 376
    Far2m,
}

impl FarHostVariant {
    /// Pick the best-matching variant for a given `StructSize`.
    pub fn from_struct_size(size: i32) -> Self {
        match size {
            260 => Self::Far2l26,
            276 => Self::Far2l28,
            376 => Self::Far2m,
            // Future-proof fallback: pick the nearest known layout
            s if s < 260 => {
                info!("FarApi: unknown StructSize {} (< 260), assuming far2l_26", s);
                Self::Far2l26
            }
            s if s < 276 => {
                info!("FarApi: unknown StructSize {}, assuming far2l_26", s);
                Self::Far2l26
            }
            s if s < 376 => {
                info!("FarApi: unknown StructSize {}, assuming far2l_28", s);
                Self::Far2l28
            }
            s => {
                info!("FarApi: unknown StructSize {} (> 376), assuming far2m", s);
                Self::Far2m
            }
        }
    }
}

// ── Byte offsets ────────────────────────────────────────────────────────
//
// Common prefix (identical for all variants, offsets 0..172):
//
//   0: StructSize (i32, 4 bytes)
//   4: ModuleName (pointer, 8 bytes)
//  12: ModuleNumber (IntPtr, 8 bytes)
//  20: RootKey (pointer, 8 bytes)
//  28: Menu
//  36: Message
//  44: GetMsg
//  52: Control
//  60..164: SaveScreen .. AdvControl
//
// After offset 172 the layout diverges per variant.

const OFF_MODULE_NAME: usize = 4;
const OFF_MODULE_NUMBER: usize = 12;
const OFF_ROOT_KEY: usize = 20;
const OFF_MENU: usize = 28;
const OFF_MESSAGE: usize = 36;
const OFF_GET_MSG: usize = 44;
const OFF_CONTROL: usize = 52;

/// Variant-specific offsets for fields that sit *after* the common prefix.
struct ExtOffsets {
    dialog_init: usize,
    dialog_run: usize,
    dialog_free: usize,
    send_dlg_message: usize,
    def_dlg_proc: usize,
}

/// far2l 2.6.0 (260 bytes total, NO AdvControlAsync):
/// 172=InputBox, 180=DialogInit, 188=DialogRun, …
const EXT_FAR2L_26: ExtOffsets = ExtOffsets {
    dialog_init: 180,
    dialog_run: 188,
    dialog_free: 196,
    send_dlg_message: 204,
    def_dlg_proc: 212,
};

/// far2l 2.8.0 (276 bytes total, HAS AdvControlAsync at 172):
/// 180=InputBox, 188=DialogInit, 196=DialogRun, …
const EXT_FAR2L_28: ExtOffsets = ExtOffsets {
    dialog_init: 188,
    dialog_run: 196,
    dialog_free: 204,
    send_dlg_message: 212,
    def_dlg_proc: 220,
};

/// far2m (376 bytes total). Assumed same as far2l 2.8 for these core fields,
/// with extra V3 fields appended after offset 276.
const EXT_FAR2M: ExtOffsets = ExtOffsets {
    dialog_init: 188,
    dialog_run: 196,
    dialog_free: 204,
    send_dlg_message: 212,
    def_dlg_proc: 220,
};

// ── FarApi ─────────────────────────────────────────────────────────────

/// Runtime-discovered API: holds function pointers extracted from the raw
/// `PluginStartupInfo` buffer by correct byte offsets for the detected host.
pub struct FarApi {
    pub variant: FarHostVariant,
    pub struct_size: i32,

    // ── Common-prefix fields (same offset for all variants) ────────
    pub module_number: IntPtr,
    pub module_name: *const WCHAR,
    pub root_key: *const WCHAR,
    pub get_msg: FARAPIGETMSG,
    pub menu: FARAPIMENU,
    pub message: FARAPIMESSAGE,
    pub control: FARAPICONTROL,

    // ── Variant-specific fields ────────────────────────────────────
    pub dialog_init: FARAPIDIALOGINIT,
    pub dialog_run: FARAPIDIALOGRUN,
    pub dialog_free: FARAPIDIALOGFREE,
    pub send_dlg_message: FARAPISENDDLGMESSAGE,
    pub def_dlg_proc: FARAPIDEFDLGPROC,
}

/// Read a `T`-sized value from `base + offset` without any alignment
/// requirement.  All fields in `PluginStartupInfo` are either 4 or 8
/// bytes so this is always safe provided `offset + size_of::<T>()`
/// does not exceed the allocation.
#[inline]
unsafe fn read_at<T: Copy>(base: *const u8, offset: usize) -> T {
    std::ptr::read_unaligned(base.add(offset) as *const T)
}

impl FarApi {
    /// Build a `FarApi` from the raw pointer that the host passed to
    /// `SetStartupInfoW`.  Returns `None` only when `info` is null.
    pub unsafe fn from_raw_info(info: *const PluginStartupInfo) -> Option<Self> {
        if info.is_null() {
            return None;
        }
        let base = info as *const u8;

        // ── 1. Read StructSize and detect variant ──────────────────
        let struct_size: i32 = read_at(base, 0);

        if struct_size < 200 {
            info!("FarApi: StructSize {} is too small, cannot parse safely", struct_size);
            return None;
        }

        // Detect if the struct is packed(2) or unpacked (default 8-byte alignment).
        // Packed means there is no padding after `StructSize` (offset 4).
        // Unpacked means there are 4 bytes of padding (offset 8).
        // Unpacked structs were used in older far2l builds (e.g. some 2.6 versions) on Linux x64.
        let is_unpacked = match struct_size {
            264 | 280 | 388 | 380 => true, // include possible unpacked sizes
            _ => false,
        };

        let pad = if is_unpacked { 4 } else { 0 };

        let variant = FarHostVariant::from_struct_size(if is_unpacked { struct_size - 4 } else { struct_size });

        info!(
            "FarApi: StructSize={}, variant={:?}, our_struct={}, unpacked={}",
            struct_size,
            variant,
            std::mem::size_of::<PluginStartupInfo>(),
            is_unpacked
        );

        // ── 2. Read common-prefix fields ───────────────────────────
        let module_name: *const WCHAR = read_at(base, OFF_MODULE_NAME + pad);
        let module_number: IntPtr = read_at(base, OFF_MODULE_NUMBER + pad);
        let root_key: *const WCHAR = read_at(base, OFF_ROOT_KEY + pad);
        let get_msg: FARAPIGETMSG = read_at(base, OFF_GET_MSG + pad);
        let menu: FARAPIMENU = read_at(base, OFF_MENU + pad);
        let message: FARAPIMESSAGE = read_at(base, OFF_MESSAGE + pad);
        let control: FARAPICONTROL = read_at(base, OFF_CONTROL + pad);

        // ── 3. Read variant-specific fields ────────────────────────
        let ext = match variant {
            FarHostVariant::Far2l26 => &EXT_FAR2L_26,
            FarHostVariant::Far2l28 => &EXT_FAR2L_28,
            FarHostVariant::Far2m => &EXT_FAR2M,
        };

        let size = struct_size as usize;

        // Only read a field if its offset fits inside the struct the host gave us.
        // Special case for older far2m: if struct_size is 376, AdvControlAsync is missing,
        // so all subsequent offsets are shifted by -8 bytes.
        let far2m_shift: isize = if variant == FarHostVariant::Far2m && struct_size <= 376 { -8 } else { 0 };

        let get_offset = |ext_off: usize| -> usize {
            (ext_off as isize + far2m_shift + pad as isize) as usize
        };

        let dialog_init: FARAPIDIALOGINIT = if get_offset(ext.dialog_init) + 8 <= size {
            read_at(base, get_offset(ext.dialog_init))
        } else {
            None
        };
        let dialog_run: FARAPIDIALOGRUN = if get_offset(ext.dialog_run) + 8 <= size {
            read_at(base, get_offset(ext.dialog_run))
        } else {
            None
        };
        let dialog_free: FARAPIDIALOGFREE = if get_offset(ext.dialog_free) + 8 <= size {
            read_at(base, get_offset(ext.dialog_free))
        } else {
            None
        };
        let send_dlg_message: FARAPISENDDLGMESSAGE = if get_offset(ext.send_dlg_message) + 8 <= size {
            read_at(base, get_offset(ext.send_dlg_message))
        } else {
            None
        };
        let def_dlg_proc: FARAPIDEFDLGPROC = if get_offset(ext.def_dlg_proc) + 8 <= size {
            read_at(base, get_offset(ext.def_dlg_proc))
        } else {
            None
        };

        // ── 4. Diagnostic logging ──────────────────────────────────
        info!(
            "FarApi: GetMsg={}, Message={}, Control={}, DialogInit={}, DialogRun={}, DialogFree={}, SendDlgMsg={}",
            get_msg.is_some(),
            message.is_some(),
            control.is_some(),
            dialog_init.is_some(),
            dialog_run.is_some(),
            dialog_free.is_some(),
            send_dlg_message.is_some(),
        );

        Some(FarApi {
            variant,
            struct_size,
            module_number,
            module_name,
            root_key,
            get_msg,
            menu,
            message,
            control,
            dialog_init,
            dialog_run,
            dialog_free,
            send_dlg_message,
            def_dlg_proc,
        })
    }
}
