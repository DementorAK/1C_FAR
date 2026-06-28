#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::c_void;
use std::os::raw::c_long;

pub type HANDLE = *mut c_void;
pub type BOOL = i32;
pub type WORD = u16;
pub type DWORD = u32;
pub type BYTE = u8;
pub type WCHAR = u32; // far2m uses UTF-32
pub type IntPtr = isize;
pub type UIntPtr = usize;

pub const PANEL_NONE: HANDLE = -1isize as HANDLE;
pub const PANEL_ACTIVE: HANDLE = -1isize as HANDLE;
pub const PANEL_PASSIVE: HANDLE = -2isize as HANDLE;

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}

// ── OpenPluginInfo ─────────────────────────────────────────────────────

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct OpenPluginInfo {
    pub StructSize: i32,
    pub Flags: DWORD,
    pub HostFile: *const WCHAR,
    pub CurDir: *const WCHAR,
    pub Format: *const WCHAR,
    pub PanelTitle: *const WCHAR,
    pub InfoLines: *const InfoPanelLine,
    pub InfoLinesNumber: i32,
    pub DescrFiles: *const *const WCHAR,
    pub DescrFilesNumber: i32,
    pub PanelModesArray: *const PanelMode,
    pub PanelModesNumber: i32,
    pub StartPanelMode: i32,
    pub StartSortMode: i32,
    pub StartSortOrder: i32,
    pub KeyBar: *const KeyBarTitles,
    pub ShortcutData: *const WCHAR,
    pub CurURL: *const WCHAR,
    pub Reserved: c_long,
}

impl Default for OpenPluginInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// ── InfoPanelLine / PanelMode / KeyBarTitles (FAR 2) ───────────────────

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
pub struct InfoPanelLine {
    pub Text: *const WCHAR,
    pub Data: *const WCHAR,
    pub Separator: i32,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
pub struct PanelMode {
    pub ColumnTypes: *const WCHAR,
    pub ColumnWidths: *const WCHAR,
    pub ColumnTitles: *const *const WCHAR,
    pub FullScreen: i32,
    pub DetailedStatus: i32,
    pub AlignExtensions: i32,
    pub CaseConversion: i32,
    pub StatusColumnTypes: *const WCHAR,
    pub StatusColumnWidths: *const WCHAR,
    pub Reserved: [DWORD; 2],
}

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct KeyBarTitles {
    pub Titles: [*mut WCHAR; 12],
    pub CtrlTitles: [*mut WCHAR; 12],
    pub AltTitles: [*mut WCHAR; 12],
    pub ShiftTitles: [*mut WCHAR; 12],
    pub CtrlShiftTitles: [*mut WCHAR; 12],
    pub AltShiftTitles: [*mut WCHAR; 12],
    pub CtrlAltTitles: [*mut WCHAR; 12],
}

// ── PluginPanelItem ────────────────────────────────────────────────────

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct FAR_FIND_DATA {
    pub ftCreationTime: FILETIME,
    pub ftLastAccessTime: FILETIME,
    pub ftLastWriteTime: FILETIME,
    pub nPhysicalSize: u64,
    pub nFileSize: u64,
    pub dwFileAttributes: DWORD,
    pub dwUnixMode: DWORD,
    pub lpwszFileName: *const WCHAR,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct PluginPanelItem {
    pub FindData: FAR_FIND_DATA,
    pub UserData: DWORD_PTR,
    pub Flags: DWORD,
    pub NumberOfLinks: DWORD,
    pub Description: *const WCHAR,
    pub Owner: *const WCHAR,
    pub Group: *const WCHAR,
    pub CustomColumnData: *const *const WCHAR,
    pub CustomColumnNumber: i32,
    pub CRC32: DWORD,
    pub Reserved: [DWORD_PTR; 2],
}

impl Default for PluginPanelItem {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

// ── PanelInfo ──────────────────────────────────────────────────────────

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
pub struct PanelInfo {
    pub PanelType: i32,
    pub Plugin: i32,
    pub PanelRect: RECT,
    pub ItemsNumber: i32,
    pub SelectedItemsNumber: i32,
    pub CurrentItem: i32,
    pub TopPanelItem: i32,
    pub Visible: i32,
    pub Focus: i32,
    pub ViewMode: i32,
    pub SortMode: i32,
    pub Flags: DWORD,
    pub Reserved: DWORD,
    pub PluginHandle: HANDLE,
    pub OwnerHandle: HANDLE,
    pub OwnerID: DWORD,
}

pub type FILE_CONTROL_COMMANDS = i32;
pub const FCTL_CLOSEPLUGIN: FILE_CONTROL_COMMANDS = 0;
pub const FCTL_GETPANELINFO: FILE_CONTROL_COMMANDS = 1;
pub const FCTL_UPDATEPANEL: FILE_CONTROL_COMMANDS = 2;
pub const FCTL_REDRAWPANEL: FILE_CONTROL_COMMANDS = 3;
pub const FCTL_SETVIEWMODE: FILE_CONTROL_COMMANDS = 7;
pub const FCTL_SETPANELDIR: FILE_CONTROL_COMMANDS = 10;
pub const FCTL_SETSORTMODE: FILE_CONTROL_COMMANDS = 13;
pub const FCTL_SETSORTORDER: FILE_CONTROL_COMMANDS = 14;
pub const FCTL_GETPANELITEM: FILE_CONTROL_COMMANDS = 22;
pub const FCTL_GETSELECTEDPANELITEM: FILE_CONTROL_COMMANDS = 23;
pub const FCTL_GETCURRENTPANELITEM: FILE_CONTROL_COMMANDS = 24;
pub const FCTL_GETPANELDIR: FILE_CONTROL_COMMANDS = 25;

pub type FARMESSAGEFLAGS = u32;
pub const FMSG_WARNING: FARMESSAGEFLAGS = 0x00000001;
pub const FMSG_ERRORTYPE: FARMESSAGEFLAGS = 0x00000002;
pub const FMSG_KEEPBACKGROUND: FARMESSAGEFLAGS = 0x00000004;
pub const FMSG_LEFTALIGN: FARMESSAGEFLAGS = 0x00000010;
pub const FMSG_MB_OK: FARMESSAGEFLAGS = 0x00010000;
pub const FMSG_MB_OKCANCEL: FARMESSAGEFLAGS = 0x00020000;

pub type EDITOR_FLAGS = u32;
pub const EF_NONMODAL: EDITOR_FLAGS = 0x00000001;

pub type VIEWER_FLAGS = u32;
pub const VF_NONMODAL: VIEWER_FLAGS = 0x00000001;

#[macro_export]
macro_rules! wstr {
    ($s:expr) => {
        $crate::far::string_utils::to_wide($s)
    };
}

pub unsafe fn from_wide_ptr(ptr: *const WCHAR) -> String {
    crate::far::string_utils::from_wide_ptr(ptr)
}

// ── Basic types ────────────────────────────────────────────────────────

pub type DWORD_PTR = usize;

pub type FARAPIMESSAGE = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        Flags: DWORD,
        HelpTopic: *const WCHAR,
        Items: *const *const WCHAR,
        ItemsNumber: i32,
        ButtonsNumber: i32,
    ) -> i32,
>;

pub type FARAPIMENU = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        X: i32,
        Y: i32,
        MaxHeight: i32,
        Flags: DWORD,
        Title: *const WCHAR,
        Bottom: *const WCHAR,
        HelpTopic: *const WCHAR,
        BreakKeys: *const i32,
        BreakCode: *mut i32,
        Item: *const FarMenuItem,
        ItemsNumber: i32,
    ) -> i32,
>;

pub type FARAPIGETMSG =
    Option<unsafe extern "system" fn(PluginNumber: IntPtr, MsgId: i32) -> *const WCHAR>;

pub type FARAPICONTROL = Option<
    unsafe extern "system" fn(hPlugin: HANDLE, Command: i32, Param1: i32, Param2: LONG_PTR) -> i32,
>;

pub type FARAPISAVESCREEN =
    Option<unsafe extern "system" fn(X1: i32, Y1: i32, X2: i32, Y2: i32) -> HANDLE>;

pub type FARAPIRESTORESCREEN = Option<unsafe extern "system" fn(hScreen: HANDLE)>;

pub type FARAPIGETDIRLIST = Option<
    unsafe extern "system" fn(
        Dir: *const WCHAR,
        pPanelItem: *mut *mut FAR_FIND_DATA,
        pItemsNumber: *mut i32,
    ) -> i32,
>;

pub type FARAPIGETPLUGINDIRLIST = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        hPlugin: HANDLE,
        Dir: *const WCHAR,
        pPanelItem: *mut *mut PluginPanelItem,
        pItemsNumber: *mut i32,
    ) -> i32,
>;

pub type FARAPIFREEDIRLIST =
    Option<unsafe extern "system" fn(PanelItem: *mut FAR_FIND_DATA, nItemsNumber: i32)>;

pub type FARAPIFREEPLUGINDIRLIST =
    Option<unsafe extern "system" fn(PanelItem: *mut PluginPanelItem, nItemsNumber: i32)>;

pub type FARAPIVIEWER = Option<
    unsafe extern "system" fn(
        FileName: *const WCHAR,
        Title: *const WCHAR,
        X1: i32,
        Y1: i32,
        X2: i32,
        Y2: i32,
        Flags: DWORD,
        CodePage: u32,
    ) -> i32,
>;

pub type FARAPIEDITOR = Option<
    unsafe extern "system" fn(
        FileName: *const WCHAR,
        Title: *const WCHAR,
        X1: i32,
        Y1: i32,
        X2: i32,
        Y2: i32,
        Flags: DWORD,
        StartLine: i32,
        StartChar: i32,
        CodePage: u32,
    ) -> i32,
>;

pub type FARAPICMPNAME = Option<
    unsafe extern "system" fn(Pattern: *const WCHAR, String: *const WCHAR, SkipPath: i32) -> i32,
>;

pub type FARAPITEXT =
    Option<unsafe extern "system" fn(X: i32, Y: i32, Color: u64, Str: *const WCHAR)>;

pub type FARAPIEDITORCONTROL =
    Option<unsafe extern "system" fn(Command: i32, Param: *mut c_void) -> i32>;

pub type FARAPISHOWHELP = Option<
    unsafe extern "system" fn(ModuleName: *const WCHAR, Topic: *const WCHAR, Flags: DWORD) -> BOOL,
>;

pub type FARAPIADVCONTROL = Option<
    unsafe extern "system" fn(
        ModuleNumber: IntPtr,
        Command: i32,
        Param1: *mut c_void,
        Param2: *mut c_void,
    ) -> IntPtr,
>;

pub type FARAPIINPUTBOX = Option<
    unsafe extern "system" fn(
        Title: *const WCHAR,
        SubTitle: *const WCHAR,
        HistoryName: *const WCHAR,
        SrcText: *const WCHAR,
        DestText: *mut WCHAR,
        DestLength: i32,
        HelpTopic: *const WCHAR,
        Flags: DWORD,
    ) -> i32,
>;

// Remove: FARSTANDARDFUNCTIONS definition (large struct, has its own mod)

pub type FARAPIDIALOGINIT = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        X1: i32,
        Y1: i32,
        X2: i32,
        Y2: i32,
        HelpTopic: *const WCHAR,
        Item: *mut FarDialogItem,
        ItemsNumber: u32,
        Reserved: DWORD,
        Flags: DWORD,
        DlgProc: FARWINDOWPROC,
        Param: LONG_PTR,
    ) -> HANDLE,
>;

pub type FARAPIDIALOGRUN = Option<unsafe extern "system" fn(hDlg: HANDLE) -> i32>;

pub type FARAPIDIALOGFREE = Option<unsafe extern "system" fn(hDlg: HANDLE)>;

pub type FARAPISENDDLGMESSAGE = Option<
    unsafe extern "system" fn(hDlg: HANDLE, Msg: i32, Param1: i32, Param2: LONG_PTR) -> LONG_PTR,
>;

pub type FARAPIDEFDLGPROC = Option<
    unsafe extern "system" fn(hDlg: HANDLE, Msg: i32, Param1: i32, Param2: LONG_PTR) -> LONG_PTR,
>;

pub type FARAPIVIEWERCONTROL =
    Option<unsafe extern "system" fn(Command: i32, Param: *mut c_void) -> i32>;

pub type FARAPIPLUGINSCONTROL = Option<
    unsafe extern "system" fn(hHandle: HANDLE, Command: i32, Param1: i32, Param2: LONG_PTR) -> i32,
>;

pub type FARAPIFILEFILTERCONTROL = Option<
    unsafe extern "system" fn(hHandle: HANDLE, Command: i32, Param1: i32, Param2: LONG_PTR) -> i32,
>;

pub type FARAPIREGEXPCONTROL =
    Option<unsafe extern "system" fn(hHandle: HANDLE, Command: i32, Param: LONG_PTR) -> i32>;

pub type FARAPICOLORDIALOG = Option<unsafe extern "system" fn(Flags: i32, Color: *mut u64) -> i32>;

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct FarMenuItem {
    pub Text: *const WCHAR,
    pub Selected: i32,
    pub Checked: i32,
    pub Separator: i32,
}

pub type LONG_PTR = isize;

pub type FarKey = u32;

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub union CHAR_INFO_CHAR {
    pub UnicodeChar: u64,
    pub AsciiChar: i8,
}

impl Default for CHAR_INFO_CHAR {
    fn default() -> Self {
        Self { UnicodeChar: 0 }
    }
}

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
pub struct CHAR_INFO {
    pub Char: CHAR_INFO_CHAR,
    pub Attributes: u64,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
pub struct ColorDialogData {
    pub Color: u64,
    pub Mask: u64,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
pub struct FillTextInfo {
    pub StructSize: usize,
    pub Str: *const WCHAR,
    pub Length: usize,
    pub Color: u64,
    pub Buf: *mut CHAR_INFO,
    pub nBufCells: i32,
    pub nScreenCells: i32,
}

pub type FARMENUCALLBACK =
    Option<unsafe extern "system" fn(CallbackData: *mut c_void, Pos: i32, Key: FarKey) -> i32>;

pub type FARAPIMENUV2 = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        Id: *const GUID,
        X: i32,
        Y: i32,
        MaxHeight: i32,
        Flags: DWORD,
        Title: *const WCHAR,
        Bottom: *const WCHAR,
        HelpTopic: *const WCHAR,
        BreakKeys: *const i32,
        BreakCode: *mut i32,
        Item: *const FarMenuItem,
        ItemsNumber: i32,
        Callback: FARMENUCALLBACK,
        CallbackData: *mut c_void,
    ) -> i32,
>;

pub type FARAPIMACROCONTROL = Option<
    unsafe extern "system" fn(
        PluginId: DWORD,
        Command: i32,
        Param1: i32,
        Param2: *mut c_void,
    ) -> i32,
>;

pub type FARAPIEDITORCONTROLV2 =
    Option<unsafe extern "system" fn(EditorID: i32, Command: i32, Param: *mut c_void) -> i32>;

pub type FARAPIVIEWERCONTROLV2 =
    Option<unsafe extern "system" fn(ViewerID: i32, Command: i32, Param: *mut c_void) -> i32>;

pub type FARAPIPLUGINSCONTROLV3 = Option<
    unsafe extern "system" fn(
        hHandle: HANDLE,
        Command: i32,
        Param1: isize,
        Param2: *mut c_void,
    ) -> isize,
>;

pub type FARAPICOLORDIALOGV2 = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        Data: *mut ColorDialogData,
        Flags: DWORD,
    ) -> i32,
>;

pub type FARAPIFREESCREEN = Option<unsafe extern "system" fn(hScreen: HANDLE)>;

pub type FARAPIDIALOGINITV3 = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        Id: *const GUID,
        X1: i32,
        Y1: i32,
        X2: i32,
        Y2: i32,
        HelpTopic: *const WCHAR,
        Item: *mut FarDialogItem,
        ItemsNumber: u32,
        Reserved: DWORD,
        Flags: DWORD,
        DlgProc: FARWINDOWPROC,
        Param: LONG_PTR,
    ) -> HANDLE,
>;

pub type FARAPITEXTV2 = Option<
    unsafe extern "system" fn(X: i32, Y: i32, Color: *const ColorDialogData, Str: *const WCHAR),
>;

pub type FARAPIMESSAGEV3 = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        Id: *const GUID,
        Flags: DWORD,
        HelpTopic: *const WCHAR,
        Items: *const *const WCHAR,
        ItemsNumber: i32,
        ButtonsNumber: i32,
    ) -> i32,
>;

pub type FARAPIINPUTBOXV3 = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        Id: *const GUID,
        Title: *const WCHAR,
        SubTitle: *const WCHAR,
        HistoryName: *const WCHAR,
        SrcText: *const WCHAR,
        DestText: *mut WCHAR,
        DestLength: i32,
        HelpTopic: *const WCHAR,
        Flags: DWORD,
    ) -> i32,
>;

pub type FARFILLTEXT = Option<unsafe extern "system" fn(Info: *mut FillTextInfo) -> i32>;

// FAR 2 PluginStartupInfo — must match far2m SDK layout with pack(2)
#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct PluginStartupInfo {
    pub StructSize: i32,
    pub ModuleName: *const WCHAR,
    pub ModuleNumber: IntPtr,
    pub RootKey: *const WCHAR,
    pub Menu: FARAPIMENU,
    pub Message: FARAPIMESSAGE,
    pub GetMsg: FARAPIGETMSG,
    pub Control: FARAPICONTROL,
    pub SaveScreen: FARAPISAVESCREEN,
    pub RestoreScreen: FARAPIRESTORESCREEN,
    pub GetDirList: FARAPIGETDIRLIST,
    pub GetPluginDirList: FARAPIGETPLUGINDIRLIST,
    pub FreeDirList: FARAPIFREEDIRLIST,
    pub FreePluginDirList: FARAPIFREEPLUGINDIRLIST,
    pub Viewer: FARAPIVIEWER,
    pub Editor: FARAPIEDITOR,
    pub CmpName: FARAPICMPNAME,
    pub Text: FARAPITEXT,
    pub EditorControl: FARAPIEDITORCONTROL,
    pub FSF: *mut c_void,
    pub ShowHelp: FARAPISHOWHELP,
    pub AdvControl: FARAPIADVCONTROL,
    pub AdvControlAsync: FARAPIADVCONTROL,
    pub InputBox: FARAPIINPUTBOX,
    pub DialogInit: FARAPIDIALOGINIT,
    pub DialogRun: FARAPIDIALOGRUN,
    pub DialogFree: FARAPIDIALOGFREE,
    pub SendDlgMessage: FARAPISENDDLGMESSAGE,
    pub DefDlgProc: FARAPIDEFDLGPROC,
    pub Reserved: IntPtr,
    pub ViewerControl: FARAPIVIEWERCONTROL,
    pub PluginsControl: FARAPIPLUGINSCONTROL,
    pub FileFilterControl: FARAPIFILEFILTERCONTROL,
    pub RegExpControl: FARAPIREGEXPCONTROL,
    pub MacroControl: FARAPIMACROCONTROL,
    pub EditorControlV2: FARAPIEDITORCONTROLV2,
    pub ViewerControlV2: FARAPIVIEWERCONTROLV2,
    pub PluginsControlV3: FARAPIPLUGINSCONTROLV3,
    pub ColorDialogV2: FARAPICOLORDIALOGV2,
    pub FreeScreen: FARAPIFREESCREEN,
    pub Private: *const c_void,
    pub LuafarLoaded: i32,
    pub DialogInitV3: FARAPIDIALOGINITV3,
    pub TextV2: FARAPITEXTV2,
    pub MessageV3: FARAPIMESSAGEV3,
    pub MenuV2: FARAPIMENUV2,
    pub InputBoxV3: FARAPIINPUTBOXV3,
    pub ColorDialog: FARAPICOLORDIALOG,
    pub FillText: FARFILLTEXT,
}

impl Default for PluginStartupInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

pub type OPERATION_MODES = u32;
pub const OPM_SILENT: OPERATION_MODES = 0x0001;
pub const OPM_FIND: OPERATION_MODES = 0x0002;
pub const OPM_VIEW: OPERATION_MODES = 0x0004;
pub const OPM_EDIT: OPERATION_MODES = 0x0008;
pub const OPM_TOPLEVEL: OPERATION_MODES = 0x0010;
pub const OPM_DESCR: OPERATION_MODES = 0x0020;
pub const OPM_QUICKVIEW: OPERATION_MODES = 0x0040;
pub const OPM_PGDN: OPERATION_MODES = 0x0080;
pub const OPM_COMMANDS: OPERATION_MODES = 0x0100;
pub const OPM_NONE: OPERATION_MODES = 0;

pub type OPENPLUGININFO_FLAGS = u32;
pub const OPIF_USEFILTER: OPENPLUGININFO_FLAGS = 0x00000001;
pub const OPIF_USESORTGROUPS: OPENPLUGININFO_FLAGS = 0x00000002;
pub const OPIF_USEHIGHLIGHTING: OPENPLUGININFO_FLAGS = 0x00000004;
pub const OPIF_ADDDOTS: OPENPLUGININFO_FLAGS = 0x00000008;
pub const OPIF_RAWSELECTION: OPENPLUGININFO_FLAGS = 0x00000010;
pub const OPIF_REALNAMES: OPENPLUGININFO_FLAGS = 0x00000020;
pub const OPIF_SHOWNAMESONLY: OPENPLUGININFO_FLAGS = 0x00000040;

// ── Dialog API types (FAR 2) ───────────────────────────────────────────

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct FarListItem {
    pub Flags: DWORD,
    pub Text: *const WCHAR,
    pub Reserved: [DWORD; 3],
}

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct FarList {
    pub ItemsNumber: i32,
    pub Items: *mut FarListItem,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub union FarDialogItemParam {
    pub Reserved: DWORD_PTR,
    pub Selected: i32,
    pub History: *const WCHAR,
    pub Mask: *const WCHAR,
    pub ListItems: *mut FarList,
    pub ListPos: i32,
    pub VBuf: *mut CHAR_INFO,
}

impl Default for FarDialogItemParam {
    fn default() -> Self {
        FarDialogItemParam { Reserved: 0 }
    }
}

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct FarDialogItem {
    pub Type: i32,
    pub X1: i32,
    pub Y1: i32,
    pub X2: i32,
    pub Y2: i32,
    pub Focus: i32,
    pub Param: FarDialogItemParam,
    pub Flags: DWORD,
    pub DefaultButton: i32,
    pub PtrData: *const WCHAR,
    pub MaxLen: usize,
}

impl Default for FarDialogItem {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

pub const DI_TEXT: i32 = 0;
pub const DI_VTEXT: i32 = 1;
pub const DI_SINGLEBOX: i32 = 2;
pub const DI_DOUBLEBOX: i32 = 3;
pub const DI_EDIT: i32 = 4;
pub const DI_PSWEDIT: i32 = 5;
pub const DI_FIXEDIT: i32 = 6;
pub const DI_BUTTON: i32 = 7;
pub const DI_CHECKBOX: i32 = 8;
pub const DI_RADIOBUTTON: i32 = 9;
pub const DI_COMBOBOX: i32 = 10;
pub const DI_LISTBOX: i32 = 11;
pub const DI_MEMOEDIT: i32 = 12;
pub const DI_USERCONTROL: i32 = 255;

pub const DIF_NONE: DWORD = 0;
pub const DIF_GROUP: DWORD = 0x00000400;
pub const DIF_LEFTTEXT: DWORD = 0x00000800;
pub const DIF_MOVESELECT: DWORD = 0x00001000;
pub const DIF_SHOWAMPERSAND: DWORD = 0x00002000;
pub const DIF_CENTERGROUP: DWORD = 0x00004000;
pub const DIF_NOBRACKETS: DWORD = 0x00008000;
pub const DIF_SEPARATOR: DWORD = 0x00010000;
pub const DIF_SEPARATOR2: DWORD = 0x00020000;
pub const DIF_BTNNOCLOSE: DWORD = 0x00040000;
pub const DIF_CENTERTEXT: DWORD = 0x00040000;
pub const DIF_NOFOCUS: DWORD = 0x40000000;
pub const DIF_DISABLE: DWORD = 0x80000000;

pub const DM_FIRST: i32 = 0;
pub const DM_CLOSE: i32 = 1;
pub const DM_GETDLGDATA: i32 = 4;
pub const DM_GETTEXT: i32 = 7;
pub const DM_KEY: i32 = 9;
pub const DM_REDRAW: i32 = 14;
pub const DM_GETFOCUS: i32 = 18;
pub const DM_GETTEXTPTR: i32 = 21;
pub const DM_SETTEXTPTR: i32 = 22;
pub const DM_GETCHECK: i32 = 25;
pub const DM_SETCHECK: i32 = 26;

pub const FDLG_WARNING: DWORD = 0x00000001;
pub const FDLG_SMALLDIALOG: DWORD = 0x00000002;
pub const FDLG_NODRAWSHADOW: DWORD = 0x00000004;
pub const FDLG_NODRAWPANEL: DWORD = 0x00000008;
pub const FDLG_KEEPCONSOLETITLE: DWORD = 0x00000020;
pub const FDLG_REGULARIDLE: DWORD = 0x00000040;

pub type FARWINDOWPROC = Option<
    unsafe extern "system" fn(hDlg: HANDLE, Msg: i32, Param1: i32, Param2: LONG_PTR) -> LONG_PTR,
>;

// --- Windows Registry FFI (WinPort) ---
pub type HKEY = *mut c_void;
pub type LSTATUS = i32;

pub const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as usize as HKEY;
pub const KEY_ALL_ACCESS: u32 = 0xF003F;
pub const REG_DWORD: u32 = 4;
pub const ERROR_SUCCESS: LSTATUS = 0;

extern "C" {
    #[link_name = "WINPORT_RegCreateKeyEx"]
    pub fn RegCreateKeyExW(
        hKey: HKEY,
        lpSubKey: *const u32,
        Reserved: u32,
        lpClass: *const u32,
        dwOptions: u32,
        samDesired: u32,
        lpSecurityAttributes: *mut c_void,
        phkResult: *mut HKEY,
        lpdwDisposition: *mut u32,
    ) -> LSTATUS;

    #[link_name = "WINPORT_RegSetValueEx"]
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: *const u32,
        Reserved: u32,
        dwType: u32,
        lpData: *const u8,
        cbData: u32,
    ) -> LSTATUS;

    #[link_name = "WINPORT_RegQueryValueEx"]
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: *const u32,
        lpReserved: *mut u32,
        lpType: *mut u32,
        lpData: *mut u8,
        lpcbData: *mut u32,
    ) -> LSTATUS;

    #[link_name = "WINPORT_RegCloseKey"]
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
}

// ── PluginInfo ─────────────────────────────────────────────────────────

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct PluginInfo {
    pub StructSize: i32,
    pub Flags: DWORD,
    pub DiskMenuStrings: *const *const WCHAR,
    pub Reserved0: *mut i32,
    pub DiskMenuStringsNumber: i32,
    pub PluginMenuStrings: *const *const WCHAR,
    pub PluginMenuStringsNumber: i32,
    pub PluginConfigStrings: *const *const WCHAR,
    pub PluginConfigStringsNumber: i32,
    pub CommandPrefix: *const WCHAR,
    pub SysID: DWORD,
    pub DiskMenuGuids: *const GUID,
    pub PluginMenuGuids: *const GUID,
    pub PluginConfigGuids: *const GUID,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
pub struct VersionInfo {
    pub Major: DWORD,
    pub Minor: DWORD,
    pub Revision: DWORD,
    pub Build: DWORD,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct GlobalInfo {
    pub StructSize: usize,
    pub SysID: DWORD,
    pub Version: VersionInfo,
    pub Title: *const WCHAR,
    pub Description: *const WCHAR,
    pub Author: *const WCHAR,
    pub UseMenuGuids: i32,
}

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct ConfigureInfo {
    pub StructSize: usize,
    pub Guid: *const GUID,
}

pub const OPEN_PLUGINSMENU: i32 = 1;
