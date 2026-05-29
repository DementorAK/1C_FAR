#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::ffi::c_void;

pub type HANDLE = *mut c_void;
pub type BOOL = i32;
pub type WORD = u16;
pub type DWORD = u32;
pub type BYTE = u8;
pub type WCHAR = u32; // far2l uses UTF-32
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

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
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
#[derive(Clone, Copy, Default)]
pub struct PluginPanelItem {
    pub FindData: FAR_FIND_DATA,
    pub UserData: usize,
    pub Flags: DWORD,
    pub NumberOfLinks: DWORD,
    pub Description: *const WCHAR,
    pub Owner: *const WCHAR,
    pub Group: *const WCHAR,
    pub CustomColumnData: *const *const WCHAR,
    pub CustomColumnNumber: i32,
    pub CRC32: DWORD,
    pub Reserved: [usize; 2],
}

#[repr(C, packed(2))]
#[derive(Clone, Copy, Default)]
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
}

pub type FARAPIMENU = *mut c_void;
pub type FARAPIMESSAGE = Option<
    unsafe extern "system" fn(
        PluginNumber: IntPtr,
        Flags: DWORD,
        HelpTopic: *const WCHAR,
        Items: *const *const WCHAR,
        ItemsNumber: i32,
        ButtonsNumber: i32,
    ) -> IntPtr,
>;
pub type FARAPIGETMSG =
    Option<unsafe extern "system" fn(PluginNumber: IntPtr, MsgId: i32) -> *const WCHAR>;
pub type FARAPICONTROL = Option<
    unsafe extern "system" fn(hPlugin: HANDLE, Command: i32, Param1: i32, Param2: IntPtr) -> i32,
>;
pub type FARAPISAVESCREEN = *mut c_void;
pub type FARAPIRESTORESCREEN = *mut c_void;
pub type FARAPIGETDIRLIST = *mut c_void;
pub type FARAPIGETPLUGINDIRLIST = *mut c_void;
pub type FARAPIFREEDIRLIST = *mut c_void;
pub type FARAPIFREEPLUGINDIRLIST = *mut c_void;
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
pub type FARAPICMPNAME = *mut c_void;
pub type FARAPITEXT = *mut c_void;
pub type FARAPIEDITORCONTROL = *mut c_void;
pub type FARSTANDARDFUNCTIONS = *mut c_void;
pub type FARAPISHOWHELP = *mut c_void;
pub type FARAPIADVCONTROL = *mut c_void;
pub type FARAPIINPUTBOX = *mut c_void;
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
        Param: IntPtr,
    ) -> HANDLE,
>;
pub type FARAPIDIALOGRUN =
    Option<unsafe extern "system" fn(hDlg: HANDLE) -> i32>;
pub type FARAPIDIALOGFREE =
    Option<unsafe extern "system" fn(hDlg: HANDLE)>;
pub type FARAPISENDDLGMESSAGE = Option<
    unsafe extern "system" fn(hDlg: HANDLE, Msg: i32, Param1: i32, Param2: IntPtr) -> IntPtr,
>;
pub type FARAPIDEFDLGPROC = Option<
    unsafe extern "system" fn(hDlg: HANDLE, Msg: i32, Param1: i32, Param2: IntPtr) -> IntPtr,
>;
pub type FARAPIVIEWERCONTROL = *mut c_void;
pub type FARAPIPLUGINSCONTROL = *mut c_void;
pub type FARAPIFILEFILTERCONTROL = *mut c_void;
pub type FARAPIREGEXPCONTROL = *mut c_void;
pub type FARAPICOLORDIALOG = *mut c_void;

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
    pub FSF: FARSTANDARDFUNCTIONS,
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
    pub ColorDialog: FARAPICOLORDIALOG,
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

impl Default for KeyBarTitles {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

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
    pub Reserved: IntPtr,
}

impl Default for OpenPluginInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

pub type OPENPLUGIN_OPENFROM = i32;
pub const OPEN_DISKMENU: OPENPLUGIN_OPENFROM = 0;
pub const OPEN_PLUGINSMENU: OPENPLUGIN_OPENFROM = 1;
pub const OPEN_FINDLIST: OPENPLUGIN_OPENFROM = 2;
pub const OPEN_SHORTCUT: OPENPLUGIN_OPENFROM = 3;
pub const OPEN_COMMANDLINE: OPENPLUGIN_OPENFROM = 4;
pub const OPEN_EDITOR: OPENPLUGIN_OPENFROM = 5;
pub const OPEN_VIEWER: OPENPLUGIN_OPENFROM = 6;
pub const OPEN_FILEPANEL: OPENPLUGIN_OPENFROM = 7;
pub const OPEN_DIALOG: OPENPLUGIN_OPENFROM = 8;
pub const OPEN_ANALYSE: OPENPLUGIN_OPENFROM = 9;

pub type PANELINFOFLAGS = u32;
pub const PFLAGS_SHOWHIDDEN: PANELINFOFLAGS = 0x00000001;
pub const PFLAGS_HIGHLIGHT: PANELINFOFLAGS = 0x00000002;
pub const PFLAGS_REVERSESORTORDER: PANELINFOFLAGS = 0x00000004;
pub const PFLAGS_USESORTGROUPS: PANELINFOFLAGS = 0x00000008;
pub const PFLAGS_SELECTEDFIRST: PANELINFOFLAGS = 0x00000010;
pub const PFLAGS_REALNAMES: PANELINFOFLAGS = 0x00000020;
pub const PFLAGS_NUMERICSORT: PANELINFOFLAGS = 0x00000040;
pub const PFLAGS_PANELLEFT: PANELINFOFLAGS = 0x00000080;
pub const PFLAGS_DIRECTORIESFIRST: PANELINFOFLAGS = 0x00000100;
pub const PFLAGS_USECRC32: PANELINFOFLAGS = 0x00000200;
pub const PFLAGS_CASESENSITIVESORT: PANELINFOFLAGS = 0x00000400;

pub type PANELINFOTYPE = i32;
pub const PTYPE_FILEPANEL: PANELINFOTYPE = 0;
pub const PTYPE_TREEPANEL: PANELINFOTYPE = 1;
pub const PTYPE_QVIEWPANEL: PANELINFOTYPE = 2;
pub const PTYPE_INFOPANEL: PANELINFOTYPE = 3;

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
        $crate::far::string_utils::to_wide($s).as_ptr()
    };
}

pub unsafe fn from_wide_ptr(ptr: *const WCHAR) -> String {
    crate::far::string_utils::from_wide_ptr(ptr)
}

// --- Dialog API types (FAR 2) ---

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
    pub Reserved: IntPtr,
    pub Selected: i32,
    pub History: *const WCHAR,
    pub Mask: *const WCHAR,
    pub ListItems: *mut FarList,
    pub ListPos: i32,
    pub VBuf: *mut c_void,
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
    unsafe extern "system" fn(hDlg: HANDLE, Msg: i32, Param1: i32, Param2: IntPtr) -> IntPtr,
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
