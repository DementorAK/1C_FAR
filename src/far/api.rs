#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::c_void;

// --- Windows Base Types ---
pub type HANDLE = *mut c_void;
pub type BOOL = i32;
pub type WORD = u16;
pub type DWORD = u32;
pub type BYTE = u8;
pub type WCHAR = u16;
pub type IntPtr = isize;
pub type UIntPtr = usize;
pub type COLORREF = u32;
pub type LPSECURITY_ATTRIBUTES = *mut c_void;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct COORD {
    pub X: i16,
    pub Y: i16,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct FILETIME {
    pub dwLowDateTime: u32,
    pub dwHighDateTime: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct INPUT_RECORD {
    pub EventType: WORD,
    pub Event: [u8; 18],
}

impl Default for INPUT_RECORD {
    fn default() -> Self {
        Self { EventType: 0, Event: [0; 18] }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}
pub type UUID = GUID;

// --- FAR Constants ---
pub const CP_UTF16LE: UIntPtr = 1200;
pub const CP_UTF16BE: UIntPtr = 1201;
pub const CP_UNICODE: UIntPtr = CP_UTF16LE;
pub const CP_DEFAULT: UIntPtr = !0;
pub const CP_REDETECT: UIntPtr = !1;

// --- FAR Enums ---
pub type UNDERLINE_STYLE = i32;
pub const UNDERLINE_NONE: UNDERLINE_STYLE = 0;
pub const UNDERLINE_SINGLE: UNDERLINE_STYLE = 1;
pub const UNDERLINE_DOUBLE: UNDERLINE_STYLE = 2;
pub const UNDERLINE_CURLY: UNDERLINE_STYLE = 3;
pub const UNDERLINE_DOT: UNDERLINE_STYLE = 4;
pub const UNDERLINE_DASH: UNDERLINE_STYLE = 5;

pub type FARDIALOGITEMTYPES = i32;
pub const DI_TEXT: FARDIALOGITEMTYPES = 0;
pub const DI_VTEXT: FARDIALOGITEMTYPES = 1;
pub const DI_SINGLEBOX: FARDIALOGITEMTYPES = 2;
pub const DI_DOUBLEBOX: FARDIALOGITEMTYPES = 3;
pub const DI_EDIT: FARDIALOGITEMTYPES = 4;
pub const DI_PSWEDIT: FARDIALOGITEMTYPES = 5;
pub const DI_FIXEDIT: FARDIALOGITEMTYPES = 6;
pub const DI_BUTTON: FARDIALOGITEMTYPES = 7;
pub const DI_CHECKBOX: FARDIALOGITEMTYPES = 8;
pub const DI_RADIOBUTTON: FARDIALOGITEMTYPES = 9;
pub const DI_COMBOBOX: FARDIALOGITEMTYPES = 10;
pub const DI_LISTBOX: FARDIALOGITEMTYPES = 11;
pub const DI_USERCONTROL: FARDIALOGITEMTYPES = 255;

pub type FARMESSAGE = i32;
pub const DM_FIRST: FARMESSAGE = 0;
pub const DM_CLOSE: FARMESSAGE = 1;
pub const DM_ENABLE: FARMESSAGE = 2;
pub const DM_ENABLEREDRAW: FARMESSAGE = 3;
pub const DM_GETDLGDATA: FARMESSAGE = 4;
pub const DM_GETDLGITEM: FARMESSAGE = 5;
pub const DM_GETDLGRECT: FARMESSAGE = 6;
pub const DM_GETTEXT: FARMESSAGE = 7;
pub const DM_KEY: FARMESSAGE = 9;
pub const DM_MOVEDIALOG: FARMESSAGE = 10;
pub const DM_SETDLGDATA: FARMESSAGE = 11;
pub const DM_SETDLGITEM: FARMESSAGE = 12;
pub const DM_SETFOCUS: FARMESSAGE = 13;
pub const DM_REDRAW: FARMESSAGE = 14;
pub const DM_SETTEXT: FARMESSAGE = 15;
pub const DM_SETMAXTEXTLENGTH: FARMESSAGE = 16;
pub const DM_SHOWDIALOG: FARMESSAGE = 17;
pub const DM_GETFOCUS: FARMESSAGE = 18;
pub const DM_GETCURSORPOS: FARMESSAGE = 19;
pub const DM_SETCURSORPOS: FARMESSAGE = 20;
pub const DM_SETTEXTPTR: FARMESSAGE = 22;
pub const DM_SHOWITEM: FARMESSAGE = 23;
pub const DM_ADDHISTORY: FARMESSAGE = 24;
pub const DM_GETCHECK: FARMESSAGE = 25;
pub const DM_SETCHECK: FARMESSAGE = 26;
pub const DM_SET3STATE: FARMESSAGE = 27;
pub const DM_LISTSORT: FARMESSAGE = 28;
pub const DM_LISTGETITEM: FARMESSAGE = 29;
pub const DM_LISTGETCURPOS: FARMESSAGE = 30;
pub const DM_LISTSETCURPOS: FARMESSAGE = 31;
pub const DM_LISTDELETE: FARMESSAGE = 32;
pub const DM_LISTADD: FARMESSAGE = 33;
pub const DM_LISTADDSTR: FARMESSAGE = 34;
pub const DM_LISTUPDATE: FARMESSAGE = 35;
pub const DM_LISTINSERT: FARMESSAGE = 36;
pub const DM_LISTFINDSTRING: FARMESSAGE = 37;
pub const DM_LISTINFO: FARMESSAGE = 38;
pub const DM_LISTGETDATA: FARMESSAGE = 39;
pub const DM_LISTSETDATA: FARMESSAGE = 40;
pub const DM_LISTSETTITLES: FARMESSAGE = 41;
pub const DM_LISTGETTITLES: FARMESSAGE = 42;
pub const DM_RESIZEDIALOG: FARMESSAGE = 43;
pub const DM_SETITEMPOSITION: FARMESSAGE = 44;
pub const DM_GETDROPDOWNOPENED: FARMESSAGE = 45;
pub const DM_SETDROPDOWNOPENED: FARMESSAGE = 46;
pub const DM_SETHISTORY: FARMESSAGE = 47;
pub const DM_GETITEMPOSITION: FARMESSAGE = 48;
pub const DM_SETINPUTNOTIFY: FARMESSAGE = 49;
pub const DM_EDITUNCHANGEDFLAG: FARMESSAGE = 50;
pub const DM_GETITEMDATA: FARMESSAGE = 51;
pub const DM_SETITEMDATA: FARMESSAGE = 52;
pub const DM_LISTSET: FARMESSAGE = 53;
pub const DM_GETCURSORSIZE: FARMESSAGE = 54;
pub const DM_SETCURSORSIZE: FARMESSAGE = 55;
pub const DM_LISTGETDATASIZE: FARMESSAGE = 56;
pub const DM_GETSELECTION: FARMESSAGE = 57;
pub const DM_SETSELECTION: FARMESSAGE = 58;
pub const DM_GETEDITPOSITION: FARMESSAGE = 59;
pub const DM_SETEDITPOSITION: FARMESSAGE = 60;
pub const DM_SETCOMBOBOXEVENT: FARMESSAGE = 61;
pub const DM_GETCOMBOBOXEVENT: FARMESSAGE = 62;
pub const DM_GETCONSTTEXTPTR: FARMESSAGE = 63;
pub const DM_GETDLGITEMSHORT: FARMESSAGE = 64;
pub const DM_SETDLGITEMSHORT: FARMESSAGE = 65;
pub const DM_GETDIALOGINFO: FARMESSAGE = 66;
pub const DM_GETDIALOGTITLE: FARMESSAGE = 67;

pub const DN_FIRST: FARMESSAGE = 4096;
pub const DN_BTNCLICK: FARMESSAGE = 4097;
pub const DN_CTLCOLORDIALOG: FARMESSAGE = 4098;
pub const DN_CTLCOLORDLGITEM: FARMESSAGE = 4099;
pub const DN_CTLCOLORDLGLIST: FARMESSAGE = 4100;
pub const DN_DRAWDIALOG: FARMESSAGE = 4101;
pub const DN_DRAWDLGITEM: FARMESSAGE = 4102;
pub const DN_EDITCHANGE: FARMESSAGE = 4103;
pub const DN_GOTFOCUS: FARMESSAGE = 4105;
pub const DN_HELP: FARMESSAGE = 4106;
pub const DN_HOTKEY: FARMESSAGE = 4107;
pub const DN_INITDIALOG: FARMESSAGE = 4108;
pub const DN_KILLFOCUS: FARMESSAGE = 4109;
pub const DN_LISTCHANGE: FARMESSAGE = 4110;
pub const DN_DRAGGED: FARMESSAGE = 4111;
pub const DN_RESIZECONSOLE: FARMESSAGE = 4112;
pub const DN_DRAWDIALOGDONE: FARMESSAGE = 4113;
pub const DN_LISTHOTKEY: FARMESSAGE = 4114;
pub const DN_INPUT: FARMESSAGE = 4115;
pub const DN_CONTROLINPUT: FARMESSAGE = 4116;
pub const DN_CLOSE: FARMESSAGE = 4117;
pub const DN_GETVALUE: FARMESSAGE = 4118;
pub const DN_DROPDOWNOPENED: FARMESSAGE = 4119;
pub const DN_DRAWDLGITEMDONE: FARMESSAGE = 4120;

pub const DM_USER: FARMESSAGE = 0x4000;

pub type FARCHECKEDSTATE = i32;
pub const BSTATE_UNCHECKED: FARCHECKEDSTATE = 0;
pub const BSTATE_CHECKED: FARCHECKEDSTATE = 1;
pub const BSTATE_3STATE: FARCHECKEDSTATE = 2;
pub const BSTATE_TOGGLE: FARCHECKEDSTATE = 3;

pub type PANELINFOTYPE = i32;
pub const PTYPE_FILEPANEL: PANELINFOTYPE = 0;
pub const PTYPE_TREEPANEL: PANELINFOTYPE = 1;
pub const PTYPE_QVIEWPANEL: PANELINFOTYPE = 2;
pub const PTYPE_INFOPANEL: PANELINFOTYPE = 3;

pub type OPENPANELINFO_SORTMODES = i32;
pub const SM_DEFAULT: OPENPANELINFO_SORTMODES = 0;
pub const SM_UNSORTED: OPENPANELINFO_SORTMODES = 1;
pub const SM_NAME: OPENPANELINFO_SORTMODES = 2;
pub const SM_EXT: OPENPANELINFO_SORTMODES = 3;
pub const SM_MTIME: OPENPANELINFO_SORTMODES = 4;
pub const SM_CTIME: OPENPANELINFO_SORTMODES = 5;
pub const SM_ATIME: OPENPANELINFO_SORTMODES = 6;
pub const SM_SIZE: OPENPANELINFO_SORTMODES = 7;
pub const SM_DESCR: OPENPANELINFO_SORTMODES = 8;
pub const SM_OWNER: OPENPANELINFO_SORTMODES = 9;
pub const SM_COMPRESSEDSIZE: OPENPANELINFO_SORTMODES = 10;
pub const SM_NUMLINKS: OPENPANELINFO_SORTMODES = 11;
pub const SM_NUMSTREAMS: OPENPANELINFO_SORTMODES = 12;
pub const SM_STREAMSSIZE: OPENPANELINFO_SORTMODES = 13;
pub const SM_NAMEONLY: OPENPANELINFO_SORTMODES = 14;
pub const SM_CHTIME: OPENPANELINFO_SORTMODES = 15;
pub const SM_USER: OPENPANELINFO_SORTMODES = 100000;

pub type FILE_CONTROL_COMMANDS = i32;
pub const FCTL_CLOSEPANEL: FILE_CONTROL_COMMANDS = 0;
pub const FCTL_GETPANELINFO: FILE_CONTROL_COMMANDS = 1;
pub const FCTL_UPDATEPANEL: FILE_CONTROL_COMMANDS = 2;
pub const FCTL_REDRAWPANEL: FILE_CONTROL_COMMANDS = 3;
pub const FCTL_GETCMDLINE: FILE_CONTROL_COMMANDS = 4;
pub const FCTL_SETCMDLINE: FILE_CONTROL_COMMANDS = 5;
pub const FCTL_SETSELECTION: FILE_CONTROL_COMMANDS = 6;
pub const FCTL_SETVIEWMODE: FILE_CONTROL_COMMANDS = 7;
pub const FCTL_INSERTCMDLINE: FILE_CONTROL_COMMANDS = 8;
pub const FCTL_SETUSERSCREEN: FILE_CONTROL_COMMANDS = 9;
pub const FCTL_SETPANELDIRECTORY: FILE_CONTROL_COMMANDS = 10;
pub const FCTL_SETCMDLINEPOS: FILE_CONTROL_COMMANDS = 11;
pub const FCTL_GETCMDLINEPOS: FILE_CONTROL_COMMANDS = 12;
pub const FCTL_SETSORTMODE: FILE_CONTROL_COMMANDS = 13;
pub const FCTL_SETSORTORDER: FILE_CONTROL_COMMANDS = 14;
pub const FCTL_SETCMDLINESELECTION: FILE_CONTROL_COMMANDS = 15;
pub const FCTL_GETCMDLINESELECTION: FILE_CONTROL_COMMANDS = 16;
pub const FCTL_CHECKPANELSEXIST: FILE_CONTROL_COMMANDS = 17;
pub const FCTL_GETUSERSCREEN: FILE_CONTROL_COMMANDS = 19;
pub const FCTL_ISACTIVEPANEL: FILE_CONTROL_COMMANDS = 20;
pub const FCTL_GETPANELITEM: FILE_CONTROL_COMMANDS = 21;
pub const FCTL_GETSELECTEDPANELITEM: FILE_CONTROL_COMMANDS = 22;
pub const FCTL_GETCURRENTPANELITEM: FILE_CONTROL_COMMANDS = 23;
pub const FCTL_GETPANELDIRECTORY: FILE_CONTROL_COMMANDS = 24;
pub const FCTL_GETCOLUMNTYPES: FILE_CONTROL_COMMANDS = 25;
pub const FCTL_GETCOLUMNWIDTHS: FILE_CONTROL_COMMANDS = 26;
pub const FCTL_BEGINSELECTION: FILE_CONTROL_COMMANDS = 27;
pub const FCTL_ENDSELECTION: FILE_CONTROL_COMMANDS = 28;
pub const FCTL_CLEARSELECTION: FILE_CONTROL_COMMANDS = 29;
pub const FCTL_SETDIRECTORIESFIRST: FILE_CONTROL_COMMANDS = 30;
pub const FCTL_GETPANELFORMAT: FILE_CONTROL_COMMANDS = 31;
pub const FCTL_GETPANELHOSTFILE: FILE_CONTROL_COMMANDS = 32;
pub const FCTL_GETPANELPREFIX: FILE_CONTROL_COMMANDS = 34;
pub const FCTL_SETACTIVEPANEL: FILE_CONTROL_COMMANDS = 35;

pub type OPENFROM = i32;
pub const OPEN_LEFTDISKMENU: OPENFROM = 0;
pub const OPEN_PLUGINSMENU: OPENFROM = 1;
pub const OPEN_FINDLIST: OPENFROM = 2;
pub const OPEN_SHORTCUT: OPENFROM = 3;
pub const OPEN_COMMANDLINE: OPENFROM = 4;
pub const OPEN_EDITOR: OPENFROM = 5;
pub const OPEN_VIEWER: OPENFROM = 6;
pub const OPEN_FILEPANEL: OPENFROM = 7;
pub const OPEN_DIALOG: OPENFROM = 8;
pub const OPEN_ANALYSE: OPENFROM = 9;
pub const OPEN_RIGHTDISKMENU: OPENFROM = 10;
pub const OPEN_FROMMACRO: OPENFROM = 11;
pub const OPEN_LUAMACRO: OPENFROM = 100;

pub type VERSION_STAGE = i32;
pub const VS_RELEASE: VERSION_STAGE = 0;
pub const VS_ALPHA: VERSION_STAGE = 1;
pub const VS_BETA: VERSION_STAGE = 2;
pub const VS_RC: VERSION_STAGE = 3;
pub const VS_SPECIAL: VERSION_STAGE = 4;
pub const VS_PRIVATE: VERSION_STAGE = 5;

pub type WINDOWINFO_TYPE = i32;
pub const WTYPE_UNKNOWN: WINDOWINFO_TYPE = -1;
pub const WTYPE_DESKTOP: WINDOWINFO_TYPE = 0;
pub const WTYPE_PANELS: WINDOWINFO_TYPE = 1;
pub const WTYPE_VIEWER: WINDOWINFO_TYPE = 2;
pub const WTYPE_EDITOR: WINDOWINFO_TYPE = 3;
pub const WTYPE_DIALOG: WINDOWINFO_TYPE = 4;
pub const WTYPE_VMENU: WINDOWINFO_TYPE = 5;
pub const WTYPE_HELP: WINDOWINFO_TYPE = 6;
pub const WTYPE_COMBOBOX: WINDOWINFO_TYPE = 7;
pub const WTYPE_GRABBER: WINDOWINFO_TYPE = 8;
pub const WTYPE_HMENU: WINDOWINFO_TYPE = 9;

// --- FAR Flags ---
pub type FARCOLORFLAGS = u64;
pub const FCF_FG_INDEX: FARCOLORFLAGS = 0x0000000000000001;
pub const FCF_BG_INDEX: FARCOLORFLAGS = 0x0000000000000002;
pub const FCF_FG_UNDERLINE_INDEX: FARCOLORFLAGS = 0x0000000000000008;

pub type FARMESSAGEFLAGS = u64;
pub const FMSG_WARNING: FARMESSAGEFLAGS = 0x0000000000000001;
pub const FMSG_MB_OK: FARMESSAGEFLAGS = 0x0000000000010000;

pub type PLUGIN_FLAGS = u64;
pub const PF_PRELOAD: PLUGIN_FLAGS = 0x0000000000000001;
pub const PF_DISABLEPANELS: PLUGIN_FLAGS = 0x0000000000000002;
pub const PF_EDITOR: PLUGIN_FLAGS = 0x0000000000000004;
pub const PF_VIEWER: PLUGIN_FLAGS = 0x0000000000000008;
pub const PF_FULLCMDLINE: PLUGIN_FLAGS = 0x0000000000000010;
pub const PF_DIALOG: PLUGIN_FLAGS = 0x0000000000000020;

pub type OPERATION_MODES = u64;
pub const OPM_SILENT: OPERATION_MODES = 0x0000000000000001;
pub const OPM_FIND: OPERATION_MODES = 0x0000000000000002;
pub const OPM_VIEW: OPERATION_MODES = 0x0000000000000004;
pub const OPM_EDIT: OPERATION_MODES = 0x0000000000000008;
pub const OPM_TOPLEVEL: OPERATION_MODES = 0x0000000000000010;
pub const OPM_DESCR: OPERATION_MODES = 0x0000000000000020;
pub const OPM_QUICKVIEW: OPERATION_MODES = 0x0000000000000040;

// --- FAR Structures ---
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct color_index {
    pub i: u8,
    pub reserved0: u8,
    pub reserved1: u8,
    pub a: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union FarColorValue {
    pub ForegroundColor: COLORREF,
    pub ForegroundIndex: color_index,
    pub ForegroundRGBA: rgba,
}

impl Default for FarColorValue {
    fn default() -> Self {
        Self { ForegroundColor: 0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarColor {
    pub Flags: u64,
    pub Foreground: FarColorValue,
    pub Background: FarColorValue,
    pub Underline: FarColorValue,
    pub Reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VersionInfo {
    pub Major: u32,
    pub Minor: u32,
    pub Revision: u32,
    pub Build: u32,
    pub Stage: i32, // VERSION_STAGE
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PluginMenuItem {
    pub Guids: *const GUID,
    pub Strings: *const *const u16,
    pub Count: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
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
#[derive(Clone, Copy, Default)]
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
#[derive(Clone, Copy, Default)]
pub struct FarPanelItemFreeInfo {
    pub StructSize: usize,
    pub hPlugin: HANDLE,
}

pub type FARPANELITEMFREECALLBACK = Option<unsafe extern "system" fn(UserData: *mut c_void, Info: *const FarPanelItemFreeInfo)>;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct UserDataItem {
    pub Data: *mut c_void,
    pub FreeData: FARPANELITEMFREECALLBACK,
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
    pub CustomColumnData: *const *const u16,
    pub CustomColumnNumber: usize,
    pub Flags: u64,
    pub UserData: UserDataItem,
    pub FileAttributes: UIntPtr,
    pub NumberOfLinks: UIntPtr,
    pub CRC32: UIntPtr,
    pub Reserved: [IntPtr; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PanelInfo {
    pub StructSize: usize,
    pub PluginHandle: HANDLE,
    pub OwnerGuid: GUID,
    pub Flags: u64,
    pub ItemsNumber: usize,
    pub SelectedItemsNumber: usize,
    pub PanelRect: RECT,
    pub CurrentItem: usize,
    pub TopPanelItem: usize,
    pub ViewMode: IntPtr,
    pub PanelType: i32, // PANELINFOTYPE
    pub SortMode: i32,  // OPENPANELINFO_SORTMODES
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct InfoPanelLine {
    pub Text: *const u16,
    pub Data: *const u16,
    pub Flags: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PanelMode {
    pub ColumnTypes: *const u16,
    pub ColumnWidths: *const u16,
    pub ColumnTitles: *const *const u16,
    pub StatusColumnTypes: *const u16,
    pub StatusColumnWidths: *const u16,
    pub Flags: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct KeyBarLabel {
    pub Key: FarKey,
    pub Text: *const u16,
    pub LongText: *const u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct KeyBarTitles {
    pub CountLabels: usize,
    pub Labels: *mut KeyBarLabel,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OpenPanelInfo {
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
    pub StartSortMode: i32, // OPENPANELINFO_SORTMODES
    pub StartSortOrder: IntPtr,
    pub KeyBar: *const KeyBarTitles,
    pub ShortcutData: *const u16,
    pub FreeSize: u64,
    pub UserData: UserDataItem,
    pub Instance: *mut c_void,
}

pub type GetOpenPanelInfo = OpenPanelInfo;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarKey {
    pub VirtualKeyCode: WORD,
    pub ControlKeyState: DWORD,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct AnalyseInfo {
    pub StructSize: usize,
    pub FileName: *const u16,
    pub Buffer: *mut c_void,
    pub BufferSize: usize,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OpenAnalyseInfo {
    pub StructSize: usize,
    pub Info: *mut AnalyseInfo,
    pub Handle: HANDLE,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OpenInfo {
    pub StructSize: usize,
    pub OpenFrom: i32, // OPENFROM
    pub Guid: *const GUID,
    pub Data: IntPtr,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
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
#[derive(Clone, Copy, Default)]
pub struct GetFindDataInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *mut PluginPanelItem,
    pub ItemsNumber: usize,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FreeFindDataInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *mut PluginPanelItem,
    pub ItemsNumber: usize,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ClosePanelInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CloseAnalyseInfo {
    pub StructSize: usize,
    pub Handle: HANDLE,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ExitInfo {
    pub StructSize: usize,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SetFindListInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *const PluginPanelItem,
    pub ItemsNumber: usize,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PutFilesInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *mut PluginPanelItem,
    pub ItemsNumber: usize,
    pub Move: BOOL,
    pub SrcPath: *const u16,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessHostFileInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *mut PluginPanelItem,
    pub ItemsNumber: usize,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MakeDirectoryInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub Name: *const u16,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CompareInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub Item1: *const PluginPanelItem,
    pub Item2: *const PluginPanelItem,
    pub Mode: i32, // OPENPANELINFO_SORTMODES
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GetFilesInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *mut PluginPanelItem,
    pub ItemsNumber: usize,
    pub Move: BOOL,
    pub DestPath: *const u16,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct DeleteFilesInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub PanelItem: *mut PluginPanelItem,
    pub ItemsNumber: usize,
    pub OpMode: u64,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessPanelInputInfo {
    pub StructSize: usize,
    pub hPanel: HANDLE,
    pub Rec: INPUT_RECORD,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessEditorInputInfo {
    pub StructSize: usize,
    pub Rec: INPUT_RECORD,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessConsoleInputInfo {
    pub StructSize: usize,
    pub Flags: u64,
    pub Rec: INPUT_RECORD,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OpenShortcutInfo {
    pub StructSize: usize,
    pub HostFile: *const u16,
    pub ShortcutData: *const u16,
    pub Flags: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OpenCommandLineInfo {
    pub StructSize: usize,
    pub CommandLine: *const u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OpenMacroPluginInfo {
    pub CallType: i32, // MACROCALLTYPE
    pub Data: *mut FarMacroCall,
    pub Ret: MacroPluginReturn,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct OpenMacroInfo {
    pub StructSize: usize,
    pub Count: usize,
    pub Values: *mut FarMacroValue,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ArclitePrivateInfo {
    pub StructSize: usize,
    pub CreateFile: *mut c_void,
    pub GetFileAttributes: *mut c_void,
    pub SetFileAttributes: *mut c_void,
    pub MoveFileEx: *mut c_void,
    pub DeleteFile: *mut c_void,
    pub RemoveDirectory: *mut c_void,
    pub CreateDirectory: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NetBoxPrivateInfo {
    pub StructSize: usize,
    pub CreateFile: *mut c_void,
    pub GetFileAttributes: *mut c_void,
    pub SetFileAttributes: *mut c_void,
    pub MoveFileEx: *mut c_void,
    pub DeleteFile: *mut c_void,
    pub RemoveDirectory: *mut c_void,
    pub CreateDirectory: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MacroPrivateInfo {
    pub StructSize: usize,
    pub CallFar: *mut c_void,
}

// --- Missing Event Structures ---
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessPanelEventInfo {
    pub StructSize: usize,
    pub Event: IntPtr,
    pub Param: *mut c_void,
    pub hPanel: HANDLE,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessEditorEventInfo {
    pub StructSize: usize,
    pub Event: IntPtr,
    pub Param: *mut c_void,
    pub EditorID: IntPtr,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarDialogEvent {
    pub StructSize: usize,
    pub hDlg: HANDLE,
    pub Msg: IntPtr,
    pub Param1: IntPtr,
    pub Param2: *mut c_void,
    pub Result: IntPtr,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessDialogEventInfo {
    pub StructSize: usize,
    pub Event: IntPtr,
    pub Param: *mut FarDialogEvent,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessSynchroEventInfo {
    pub StructSize: usize,
    pub Event: IntPtr,
    pub Param: *mut c_void,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ProcessViewerEventInfo {
    pub StructSize: usize,
    pub Event: IntPtr,
    pub Param: *mut c_void,
    pub ViewerID: IntPtr,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ConfigureInfo {
    pub StructSize: usize,
    pub Guid: *const GUID,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GetContentFieldsInfo {
    pub StructSize: usize,
    pub Count: usize,
    pub Names: *const *const u16,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GetContentDataInfo {
    pub StructSize: usize,
    pub FilePath: *const u16,
    pub Count: usize,
    pub Names: *const *const u16,
    pub Values: *mut *const u16,
    pub Instance: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ErrorInfo {
    pub StructSize: usize,
    pub Summary: *const u16,
    pub Description: *const u16,
}

// --- Macro API ---
pub type FARMACROVARTYPE = i32;
pub const FMVT_UNKNOWN: FARMACROVARTYPE = 0;
pub const FMVT_INTEGER: FARMACROVARTYPE = 1;
pub const FMVT_STRING: FARMACROVARTYPE = 2;
pub const FMVT_DOUBLE: FARMACROVARTYPE = 3;
pub const FMVT_BOOLEAN: FARMACROVARTYPE = 4;
pub const FMVT_BINARY: FARMACROVARTYPE = 5;
pub const FMVT_POINTER: FARMACROVARTYPE = 6;
pub const FMVT_NIL: FARMACROVARTYPE = 7;
pub const FMVT_ARRAY: FARMACROVARTYPE = 8;
pub const FMVT_PANEL: FARMACROVARTYPE = 9;
pub const FMVT_ERROR: FARMACROVARTYPE = 10;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FarMacroValue {
    pub Type: i32, // FARMACROVARTYPE
    pub Value: FarMacroValueData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union FarMacroValueData {
    pub Integer: i64,
    pub Boolean: i64,
    pub Double: f64,
    pub String: *const u16,
    pub MBString: *const i8,
    pub Pointer: *mut c_void,
    pub Binary: FarMacroBinary,
    pub Array: FarMacroArray,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FarMacroBinary {
    pub Data: *mut c_void,
    pub Size: usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FarMacroArray {
    pub Values: *mut FarMacroValue,
    pub Count: usize,
}

impl Default for FarMacroValueData {
    fn default() -> Self {
        Self { Integer: 0 }
    }
}

impl Default for FarMacroValue {
    fn default() -> Self {
        Self { Type: FMVT_NIL, Value: FarMacroValueData::default() }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarMacroCall {
    pub StructSize: usize,
    pub Count: usize,
    pub Values: *mut FarMacroValue,
    pub Callback: Option<unsafe extern "system" fn(CallbackData: *mut c_void, Values: *mut FarMacroValue, Count: usize)>,
    pub CallbackData: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MacroPluginReturn {
    pub ReturnType: IntPtr,
    pub Count: usize,
    pub Values: *mut FarMacroValue,
}

// --- Window Info ---
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct WindowInfo {
    pub StructSize: usize,
    pub Id: IntPtr,
    pub TypeName: *mut u16,
    pub Name: *mut u16,
    pub TypeNameSize: IntPtr,
    pub NameSize: IntPtr,
    pub Pos: IntPtr,
    pub Type: i32, // WINDOWINFO_TYPE
    pub Flags: u64,
}

// --- Settings API ---
pub type FARSETTINGSTYPES = i32;
pub const FST_UNKNOWN: FARSETTINGSTYPES = 0;
pub const FST_SUBKEY: FARSETTINGSTYPES = 1;
pub const FST_QWORD: FARSETTINGSTYPES = 2;
pub const FST_STRING: FARSETTINGSTYPES = 3;
pub const FST_DATA: FARSETTINGSTYPES = 4;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarSettingsCreate {
    pub StructSize: usize,
    pub Guid: GUID,
    pub Handle: HANDLE,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FarSettingsItem {
    pub StructSize: usize,
    pub Root: usize,
    pub Name: *const u16,
    pub Type: i32, // FARSETTINGSTYPES
    pub Value: FarSettingsValueData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union FarSettingsValueData {
    pub Number: u64,
    pub String: *const u16,
    pub Data: FarSettingsData,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FarSettingsData {
    pub Size: usize,
    pub Data: *const c_void,
}

impl Default for FarSettingsValueData {
    fn default() -> Self {
        Self { Number: 0 }
    }
}

// --- API Function Pointers ---
pub type FARAPIMENU = Option<unsafe extern "system" fn(PluginId: *const GUID, Id: *const GUID, X: IntPtr, Y: IntPtr, MaxHeight: IntPtr, Flags: u64, Title: *const u16, Bottom: *const u16, HelpTopic: *const u16, BreakKeys: *const FarKey, BreakCode: *mut IntPtr, Item: *const FarMenuItem, ItemsNumber: usize) -> IntPtr>;
pub type FARAPIMESSAGE = Option<unsafe extern "system" fn(PluginId: *const GUID, Id: *const GUID, Flags: u64, HelpTopic: *const u16, Items: *const *const u16, ItemsNumber: usize, ButtonsNumber: IntPtr) -> IntPtr>;
pub type FARAPIGETMSG = Option<unsafe extern "system" fn(PluginId: *const GUID, MsgId: IntPtr) -> *const u16>;
pub type FARAPIPANELCONTROL = Option<unsafe extern "system" fn(hPanel: HANDLE, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPISAVESCREEN = Option<unsafe extern "system" fn(X1: IntPtr, Y1: IntPtr, X2: IntPtr, Y2: IntPtr) -> HANDLE>;
pub type FARAPIRESTORESCREEN = Option<unsafe extern "system" fn(hScreen: HANDLE)>;
pub type FARAPIGETDIRLIST = Option<unsafe extern "system" fn(Dir: *const u16, pPanelItem: *mut *mut PluginPanelItem, pItemsNumber: *mut usize) -> IntPtr>;
pub type FARAPIGETPLUGINDIRLIST = Option<unsafe extern "system" fn(PluginId: *const GUID, hPanel: HANDLE, Dir: *const u16, pPanelItem: *mut *mut PluginPanelItem, pItemsNumber: *mut usize) -> IntPtr>;
pub type FARAPIFREEDIRLIST = Option<unsafe extern "system" fn(PanelItem: *mut PluginPanelItem, nItemsNumber: usize)>;
pub type FARAPIFREEPLUGINDIRLIST = Option<unsafe extern "system" fn(hPanel: HANDLE, PanelItem: *mut PluginPanelItem, nItemsNumber: usize)>;
pub type FARAPIVIEWER = Option<unsafe extern "system" fn(FileName: *const u16, Title: *const u16, X1: IntPtr, Y1: IntPtr, X2: IntPtr, Y2: IntPtr, Flags: u64, CodePage: UIntPtr) -> IntPtr>;
pub type FARAPIEDITOR = Option<unsafe extern "system" fn(FileName: *const u16, Title: *const u16, X1: IntPtr, Y1: IntPtr, X2: IntPtr, Y2: IntPtr, Flags: u64, StartLine: IntPtr, StartChar: IntPtr, CodePage: UIntPtr) -> IntPtr>;
pub type FARAPITEXT = Option<unsafe extern "system" fn(X: IntPtr, Y: IntPtr, Color: *const FarColor, Str: *const u16)>;
pub type FARAPIEDITORCONTROL = Option<unsafe extern "system" fn(EditorID: IntPtr, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPISHOWHELP = Option<unsafe extern "system" fn(ModuleName: *const u16, Topic: *const u16, Flags: u64) -> BOOL>;
pub type FARAPIADVCONTROL = Option<unsafe extern "system" fn(PluginId: *const GUID, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPIINPUTBOX = Option<unsafe extern "system" fn(PluginId: *const GUID, Id: *const GUID, Title: *const u16, SubTitle: *const u16, HistoryName: *const u16, SrcText: *const u16, DestText: *mut u16, DestSize: usize, HelpTopic: *const u16, Flags: u64) -> IntPtr>;
pub type FARAPICOLORDIALOG = Option<unsafe extern "system" fn(PluginId: *const GUID, Flags: u64, Color: *mut FarColor) -> BOOL>;
pub type FARAPIDIALOGINIT = Option<unsafe extern "system" fn(PluginId: *const GUID, Id: *const GUID, X1: IntPtr, Y1: IntPtr, X2: IntPtr, Y2: IntPtr, HelpTopic: *const u16, Item: *const FarDialogItem, ItemsNumber: usize, Reserved: IntPtr, Flags: u64, DlgProc: FARWINDOWPROC, Param: *mut c_void) -> HANDLE>;
pub type FARAPIDIALOGRUN = Option<unsafe extern "system" fn(hDlg: HANDLE) -> IntPtr>;
pub type FARAPIDIALOGFREE = Option<unsafe extern "system" fn(hDlg: HANDLE)>;
pub type FARAPISENDDLGMESSAGE = Option<unsafe extern "system" fn(hDlg: HANDLE, Msg: IntPtr, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPIDEFDLGPROC = Option<unsafe extern "system" fn(hDlg: HANDLE, Msg: IntPtr, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPIVIEWERCONTROL = Option<unsafe extern "system" fn(ViewerID: IntPtr, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPIPLUGINSCONTROL = Option<unsafe extern "system" fn(hHandle: HANDLE, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPIFILEFILTERCONTROL = Option<unsafe extern "system" fn(hHandle: HANDLE, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPIREGEXPCONTROL = Option<unsafe extern "system" fn(hHandle: HANDLE, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPIMACROCONTROL = Option<unsafe extern "system" fn(PluginId: *const GUID, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPISETTINGSCONTROL = Option<unsafe extern "system" fn(hHandle: HANDLE, Command: i32, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;
pub type FARAPIFREESCREEN = Option<unsafe extern "system" fn(hScreen: HANDLE)>;

pub type FARWINDOWPROC = Option<unsafe extern "system" fn(hDlg: HANDLE, Msg: IntPtr, Param1: IntPtr, Param2: *mut c_void) -> IntPtr>;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarMenuItem {
    pub Flags: u64,
    pub Text: *const u16,
    pub AccelKey: FarKey,
    pub UserData: IntPtr,
    pub Reserved: [IntPtr; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarDialogItem {
    pub Type: i32, // FARDIALOGITEMTYPES
    pub X1: IntPtr,
    pub Y1: IntPtr,
    pub X2: IntPtr,
    pub Y2: IntPtr,
    pub Param: FarDialogItemParam,
    pub History: *const u16,
    pub Mask: *const u16,
    pub Flags: u64,
    pub Data: *const u16,
    pub MaxLength: usize,
    pub UserData: IntPtr,
    pub Reserved: [IntPtr; 2],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union FarDialogItemParam {
    pub Selected: IntPtr,
    pub ListItems: *mut FarList,
    pub VBuf: *mut FAR_CHAR_INFO,
    pub Reserved0: IntPtr,
}

impl Default for FarDialogItemParam {
    fn default() -> Self {
        Self { Reserved0: 0 }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarList {
    pub StructSize: usize,
    pub ItemsNumber: usize,
    pub Items: *mut FarListItem,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarListItem {
    pub Flags: u64,
    pub Text: *const u16,
    pub UserData: IntPtr,
    pub Reserved: IntPtr,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FAR_CHAR_INFO {
    pub Char: u16,
    pub Reserved0: u16,
    pub Reserved1: i32,
    pub Attributes: FarColor,
}

// --- Standard Functions ---
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FarStandardFunctions {
    pub StructSize: usize,
    pub atoi: Option<unsafe extern "system" fn(s: *const u16) -> i32>,
    pub atoi64: Option<unsafe extern "system" fn(s: *const u16) -> i64>,
    pub itoa: Option<unsafe extern "system" fn(value: i32, Str: *mut u16, radix: i32) -> *mut u16>,
    pub itoa64: Option<unsafe extern "system" fn(value: i64, Str: *mut u16, radix: i32) -> *mut u16>,
    pub sprintf: *mut c_void,
    pub sscanf: *mut c_void,
    pub qsort: Option<unsafe extern "system" fn(base: *mut c_void, nelem: usize, width: usize, fcmp: *mut c_void, userparam: *mut c_void)>,
    pub bsearch: Option<unsafe extern "system" fn(key: *const c_void, base: *const c_void, nelem: usize, width: usize, fcmp: *mut c_void, userparam: *mut c_void) -> *mut c_void>,
    pub snprintf: *mut c_void,
    pub LIsLower: Option<unsafe extern "system" fn(Ch: u16) -> i32>,
    pub LIsUpper: Option<unsafe extern "system" fn(Ch: u16) -> i32>,
    pub LIsAlpha: Option<unsafe extern "system" fn(Ch: u16) -> i32>,
    pub LIsAlphanum: Option<unsafe extern "system" fn(Ch: u16) -> i32>,
    pub LUpper: Option<unsafe extern "system" fn(LowerChar: u16) -> u16>,
    pub LLower: Option<unsafe extern "system" fn(UpperChar: u16) -> u16>,
    pub LUpperBuf: Option<unsafe extern "system" fn(Buf: *mut u16, Length: IntPtr)>,
    pub LLowerBuf: Option<unsafe extern "system" fn(Buf: *mut u16, Length: IntPtr)>,
    pub LStrupr: Option<unsafe extern "system" fn(s1: *mut u16)>,
    pub LStrlwr: Option<unsafe extern "system" fn(s1: *mut u16)>,
    pub LStricmp: Option<unsafe extern "system" fn(s1: *const u16, s2: *const u16) -> i32>,
    pub LStrnicmp: Option<unsafe extern "system" fn(s1: *const u16, s2: *const u16, n: IntPtr) -> i32>,
    pub Unquote: Option<unsafe extern "system" fn(Str: *mut u16)>,
    pub LTrim: Option<unsafe extern "system" fn(Str: *mut u16) -> *mut u16>,
    pub RTrim: Option<unsafe extern "system" fn(Str: *mut u16) -> *mut u16>,
    pub Trim: Option<unsafe extern "system" fn(Str: *mut u16) -> *mut u16>,
    pub TruncStr: Option<unsafe extern "system" fn(Str: *mut u16, MaxLength: IntPtr) -> *mut u16>,
    pub TruncPathStr: Option<unsafe extern "system" fn(Str: *mut u16, MaxLength: IntPtr) -> *mut u16>,
    pub QuoteSpaceOnly: Option<unsafe extern "system" fn(Str: *mut u16) -> *mut u16>,
    pub PointToName: Option<unsafe extern "system" fn(Path: *const u16) -> *const u16>,
    pub GetPathRoot: Option<unsafe extern "system" fn(Path: *const u16, Root: *mut u16, DestSize: usize) -> usize>,
    pub AddEndSlash: Option<unsafe extern "system" fn(Path: *mut u16) -> BOOL>,
    pub CopyToClipboard: Option<unsafe extern "system" fn(Type: i32, Data: *const u16) -> BOOL>,
    pub PasteFromClipboard: Option<unsafe extern "system" fn(Type: i32, Data: *mut u16, Size: usize) -> usize>,
    pub FarInputRecordToName: Option<unsafe extern "system" fn(Key: *const INPUT_RECORD, KeyText: *mut u16, Size: usize) -> usize>,
    pub FarNameToInputRecord: Option<unsafe extern "system" fn(Name: *const u16, Key: *mut INPUT_RECORD) -> BOOL>,
    pub XLat: Option<unsafe extern "system" fn(Line: *mut u16, StartPos: IntPtr, EndPos: IntPtr, Flags: u64) -> *mut u16>,
    pub GetFileOwner: Option<unsafe extern "system" fn(Computer: *const u16, Name: *const u16, Owner: *mut u16, Size: usize) -> usize>,
    pub GetNumberOfLinks: Option<unsafe extern "system" fn(Name: *const u16) -> usize>,
    pub FarRecursiveSearch: Option<unsafe extern "system" fn(InitDir: *const u16, Mask: *const u16, Func: *mut c_void, Flags: u64, Param: *mut c_void)>,
    pub MkTemp: Option<unsafe extern "system" fn(Dest: *mut u16, DestSize: usize, Prefix: *const u16) -> usize>,
    pub ProcessName: Option<unsafe extern "system" fn(param1: *const u16, param2: *mut u16, size: usize, flags: u64) -> usize>,
    pub MkLink: Option<unsafe extern "system" fn(Src: *const u16, Dest: *const u16, Type: i32, Flags: u64) -> BOOL>,
    pub ConvertPath: Option<unsafe extern "system" fn(Mode: i32, Src: *const u16, Dest: *mut u16, DestSize: usize) -> usize>,
    pub GetReparsePointInfo: Option<unsafe extern "system" fn(Src: *const u16, Dest: *mut u16, DestSize: usize) -> usize>,
    pub GetCurrentDirectory: Option<unsafe extern "system" fn(Size: usize, Buffer: *mut u16) -> usize>,
    pub FormatFileSize: Option<unsafe extern "system" fn(Size: u64, Width: IntPtr, Flags: u64, Dest: *mut u16, DestSize: usize) -> usize>,
    pub FarClock: Option<unsafe extern "system" fn() -> u64>,
    pub CompareStrings: Option<unsafe extern "system" fn(Str1: *const u16, Size1: usize, Str2: *const u16, Size2: usize) -> i32>,
    pub DetectCodePage: Option<unsafe extern "system" fn(Info: *mut c_void) -> UIntPtr>,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PluginStartupInfo {
    pub StructSize: usize,
    pub ModuleName: *const u16,
    pub Menu: FARAPIMENU,
    pub Message: FARAPIMESSAGE,
    pub GetMsg: FARAPIGETMSG,
    pub PanelControl: FARAPIPANELCONTROL,
    pub SaveScreen: FARAPISAVESCREEN,
    pub RestoreScreen: FARAPIRESTORESCREEN,
    pub GetDirList: FARAPIGETDIRLIST,
    pub GetPluginDirList: FARAPIGETPLUGINDIRLIST,
    pub FreeDirList: FARAPIFREEDIRLIST,
    pub FreePluginDirList: FARAPIFREEPLUGINDIRLIST,
    pub Viewer: FARAPIVIEWER,
    pub Editor: FARAPIEDITOR,
    pub Text: FARAPITEXT,
    pub EditorControl: FARAPIEDITORCONTROL,
    pub FSF: *mut FarStandardFunctions,
    pub ShowHelp: FARAPISHOWHELP,
    pub AdvControl: FARAPIADVCONTROL,
    pub InputBox: FARAPIINPUTBOX,
    pub ColorDialog: FARAPICOLORDIALOG,
    pub DialogInit: FARAPIDIALOGINIT,
    pub DialogRun: FARAPIDIALOGRUN,
    pub DialogFree: FARAPIDIALOGFREE,
    pub SendDlgMessage: FARAPISENDDLGMESSAGE,
    pub DefDlgProc: FARAPIDEFDLGPROC,
    pub ViewerControl: FARAPIVIEWERCONTROL,
    pub PluginsControl: FARAPIPLUGINSCONTROL,
    pub FileFilterControl: FARAPIFILEFILTERCONTROL,
    pub RegExpControl: FARAPIREGEXPCONTROL,
    pub MacroControl: FARAPIMACROCONTROL,
    pub SettingsControl: FARAPISETTINGSCONTROL,
    pub Private: *const c_void,
    pub Instance: *mut c_void,
    pub FreeScreen: FARAPIFREESCREEN,
}

// --- Helper Macros and Functions ---
#[macro_export]
macro_rules! wstr {
    ($s:expr) => {
        $crate::far::api::to_wide($s).as_ptr()
    };
}

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
