use windows_sys::Win32::Foundation::HWND;

// controller <-> ui
pub enum UiEvent {
    HwndReady(HWND),
}

pub enum UiCommand {
    ForcusInput,
}

pub struct UiEndpoint {
    pub tx: std::sync::mpsc::Sender<UiEvent>,
    pub rx: std::sync::mpsc::Receiver<UiCommand>,
}

pub struct UiHandle {
    pub rx: std::sync::mpsc::Receiver<UiEvent>,
    pub tx: std::sync::mpsc::Sender<UiCommand>,
}

pub fn create_ui_endpoints() -> (UiEndpoint, UiHandle) {
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();

    (
        UiEndpoint {
            rx: cmd_rx,
            tx: event_tx,
        },
        UiHandle {
            tx: cmd_tx,
            rx: event_rx,
        },
    )
}

// controller <-> hotkey
pub enum HotkeyEvent {
    Toggle,
    RegisterResult(bool),
}

pub enum HotkeyCmd {
    Register(HWND),
    Finish,
}

pub struct HotkeyEndpoint {
    pub tx: std::sync::mpsc::Sender<HotkeyEvent>,
    pub rx: std::sync::mpsc::Receiver<HotkeyCmd>,
}

pub struct HotkeyHandle {
    pub rx: std::sync::mpsc::Receiver<HotkeyEvent>,
    pub tx: std::sync::mpsc::Sender<HotkeyCmd>,
}

pub fn create_hotkey_endpoints() -> (HotkeyEndpoint, HotkeyHandle) {
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let (handle_tx, handle_rx) = std::sync::mpsc::channel();
    (
        HotkeyEndpoint {
            tx: event_tx,
            rx: handle_rx,
        },
        HotkeyHandle {
            rx: event_rx,
            tx: handle_tx,
        },
    )
}

// controller <- tasktray
// 現状は表示・終了のコマンドの単方向通信

pub enum TrayEvent {
    ShowWindow,
    Quit,
}

pub enum TrayCmd {
    Finish,
}

pub struct TrayHandle {
    pub rx: std::sync::mpsc::Receiver<TrayEvent>,
    pub tx: std::sync::mpsc::Sender<TrayCmd>,
}

pub struct TrayEndpoint {
    pub tx: std::sync::mpsc::Sender<TrayEvent>,
    pub rx: std::sync::mpsc::Receiver<TrayCmd>,
}

pub fn create_tray_endpoints() -> (TrayEndpoint, TrayHandle) {
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    (
        TrayEndpoint {
            tx: event_tx,
            rx: cmd_rx,
        },
        TrayHandle {
            rx: event_rx,
            tx: cmd_tx,
        },
    )
}
