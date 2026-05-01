#![allow(non_snake_case)]

use std::ffi::c_void;

pub type HANDLE = *mut c_void;
pub type IntPtr = isize;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VersionInfo {
    pub Major: u32,
    pub Minor: u32,
    pub Revision: u32,
    pub Build: u32,
    pub Stage: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct UserDataItem {
    pub Data: *mut c_void,
    pub FreeData: *const c_void,
}

#[repr(C)]
pub struct GlobalInfo {
    pub StructSize: usize,
    pub MinFarVersion: VersionInfo,
    pub Version: VersionInfo,
    pub Guid: GUID,
    pub Title: *const u16,
    pub Description: *const u16,
    pub Author: *const u16,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PluginMenuItem {
    pub Guids: *const GUID,
    pub Strings: *const *const u16,
    pub Count: usize,
}

#[repr(C)]
pub struct PluginInfo {
    pub StructSize: usize,
    pub Flags: u64,
    pub DiskMenu: PluginMenuItem,
    pub PluginMenu: PluginMenuItem,
    pub PluginConfig: PluginMenuItem,
    pub CommandPrefix: *const u16,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FarStandardFunctions {
    pub StructSize: usize,
    // Incomplete, only defining what's needed for now
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PluginStartupInfo {
    pub StructSize: usize,
    pub ModuleName: *const u16,
    pub Menu: *mut c_void,
    pub Message: *mut c_void,
    pub GetMsg: *mut c_void,
    pub PanelControl: *mut c_void,
    pub SaveScreen: *mut c_void,
    pub RestoreScreen: *mut c_void,
    pub GetDirList: *mut c_void,
    pub GetPluginDirList: *mut c_void,
    pub FreeDirList: *mut c_void,
    pub FreePluginDirList: *mut c_void,
    pub Viewer: *mut c_void,
    pub Editor: *mut c_void,
    pub Text: *mut c_void,
    pub EditorControl: *mut c_void,
    pub FSF: *mut FarStandardFunctions,
    pub ShowHelp: *mut c_void,
    pub AdvControl: *mut c_void,
    pub InputBox: *mut c_void,
    pub ColorDialog: *mut c_void,
    pub DialogInit: *mut c_void,
    pub DialogRun: *mut c_void,
    pub DialogFree: *mut c_void,
    pub SendDlgMessage: *mut c_void,
    pub DefDlgProc: *mut c_void,
    pub ViewerControl: *mut c_void,
    pub PluginsControl: *mut c_void,
    pub FileFilterControl: *mut c_void,
    pub RegExpControl: *mut c_void,
    pub MacroControl: *mut c_void,
    pub SettingsControl: *mut c_void,
    pub Private: *const c_void,
    pub Instance: *mut c_void,
    pub FreeScreen: *mut c_void,
}

#[repr(C)]
pub struct OpenInfo {
    pub StructSize: usize,
    pub OpenFrom: u32,
    pub Guid: *const GUID,
    pub Data: IntPtr,
    pub Instance: *mut c_void,
}

#[repr(C)]
pub struct OpenAnalyseInfo {
    pub StructSize: usize,
    pub Info: *const AnalyseInfo,
    pub Handle: HANDLE,
}

#[repr(C)]
pub struct AnalyseInfo {
    pub StructSize: usize,
    pub FileName: *const u16,
    pub Buffer: *mut c_void,
    pub BufferSize: usize,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

// MACROS and utilities for Wide Strings
#[macro_export]
macro_rules! wstr {
    ($s:expr) => {
        $crate::far::api::to_wide($s).as_ptr()
    };
}

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FARPANELITEMUSERDATA {
    pub Data: *mut c_void,
    pub FreeData: *const c_void, // Actually a function pointer
}

impl Default for FARPANELITEMUSERDATA {
    fn default() -> Self {
        Self {
            Data: std::ptr::null_mut(),
            FreeData: std::ptr::null(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PluginPanelItem {
    pub CreationTime: FILETIME,
    pub LastAccessTime: FILETIME,
    pub LastWriteTime: FILETIME,
    pub ChangeTime: FILETIME,
    pub FileSize: u64,
    pub AllocationSize: u64,
    pub FileName: *const u16,
    pub AlternateFileName: *const u16,
    pub Description: *const u16,
    pub Owner: *const u16,
    pub CustomColumnData: *mut *const u16,
    pub CustomColumnNumber: usize,
    pub Flags: u64,
    pub UserData: UserDataItem,
    pub FileAttributes: usize,
    pub NumberOfLinks: usize,
    pub CRC32: usize,
    pub Reserved: [IntPtr; 2],
}

#[repr(C)]
pub struct GetFindDataInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *mut PluginPanelItem,
    pub ItemsNumber: usize,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

#[repr(C)]
pub struct FreeFindDataInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *mut PluginPanelItem,
    pub ItemsNumber: usize,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SetDirectoryInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub Dir: *const u16,
    pub Reserved: IntPtr,
    pub OpMode: u64,
    pub UserData: UserDataItem,
    pub Instance: *mut c_void,
}

#[repr(C)]
pub struct InfoPanelLine {
    pub Text: *const u16,
    pub Data: *const u16,
    pub Flags: u64,
}

#[repr(C)]
pub struct PanelMode {
    pub ColumnTypes: *const u16,
    pub ColumnWidths: *const u16,
    pub ColumnTitles: *const *const u16,
    pub FullScreen: IntPtr,
    pub DetailedStatus: IntPtr,
    pub AlignExtensions: IntPtr,
    pub CaseConversion: IntPtr,
    pub StatusColumnTypes: *const u16,
    pub StatusColumnWidths: *const u16,
    pub Reserved: [IntPtr; 2],
}

#[repr(C)]
pub struct KeyBarTitles {
    pub Titles: [*const u16; 12],
    pub CtrlTitles: [*const u16; 12],
    pub AltTitles: [*const u16; 12],
    pub ShiftTitles: [*const u16; 12],
    pub CtrlShiftTitles: [*const u16; 12],
    pub AltShiftTitles: [*const u16; 12],
    pub CtrlAltTitles: [*const u16; 12],
}

#[repr(C)]
pub struct GetOpenPanelInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub Flags: u64,
    pub HostFile: *const u16,
    pub CurDir: *const u16,
    pub Format: *const u16,
    pub PanelTitle: *const u16,
    pub InfoLines: *const InfoPanelLine,
    pub InfoLinesNumber: usize,
    pub DescrFiles: *const *const u16,
    pub DescrFilesNumber: usize,
    pub PanelModesArray: *const PanelMode,
    pub PanelModesNumber: usize,
    pub StartPanelMode: usize,
    pub StartSortMode: u32,
    pub StartSortOrder: u32,
    pub KeyBar: *const KeyBarTitles,
    pub ShortcutData: *const u16,
    pub FreeSize: u64,
    pub UserData: UserDataItem,
    pub Instance: *mut c_void,
}

#[repr(C)]
pub struct CloseAnalyseInfo {
    pub StructSize: usize,
    pub Handle: HANDLE,
    pub Instance: *mut c_void,
}

#[repr(C)]
pub struct ClosePanelInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub Instance: *mut c_void,
}
