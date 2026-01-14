use crate::app::endpoint;
use log::{info, warn};
use std::sync::mpsc;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsWindowVisible, PostMessageW, SW_HIDE, SW_SHOW, SetForegroundWindow, ShowWindow, WM_CLOSE,
};

struct ControllerState {
    hwnd: Option<HWND>,
    hotkey_registered: bool,
}
impl ControllerState {
    pub fn new() -> Self {
        Self {
            hwnd: None,
            hotkey_registered: false,
        }
    }
}

pub struct Controller {
    state: ControllerState,
    ui: endpoint::UiHandle,
    hotkey: endpoint::HotkeyHandle,
    tray: endpoint::TrayHandle,
    finish_rx: mpsc::Receiver<()>,
}

impl Controller {
    pub fn new(
        ui: endpoint::UiHandle,
        hotkey: endpoint::HotkeyHandle,
        tray: endpoint::TrayHandle,
        finish_rx: mpsc::Receiver<()>,
    ) -> Self {
        Self {
            state: ControllerState::new(),
            ui,
            hotkey,
            tray,
            finish_rx,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.recv_hotkey();
            self.recv_tasktray();
            self.recv_ui();
            // 終了処理
            if self.finish_rx.try_recv().is_ok() {
                // // ドロップトレイトからの処理だとうまくいかないのでここで明示的に終了処理を行う
                // info!("コントローラーの終了処理");
                let _ = self.hotkey.tx.send(endpoint::HotkeyCmd::Finish);
                let _ = self.tray.tx.send(endpoint::TrayCmd::Finish);
                break;
            }
            // CPU負荷を抑えるために短いスリープを挿入
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }

    fn recv_hotkey(&mut self) {
        if let Ok(event) = self.hotkey.rx.try_recv() {
            info!("ホットキーからのイベント受信");
            match event {
                endpoint::HotkeyEvent::Toggle => {
                    // ウィンドウの表示/非表示切り替え
                    self.toggle_window();
                }
                endpoint::HotkeyEvent::RegisterResult(success) => {
                    self.state.hotkey_registered = success;
                    if success {
                        info!("ホットキー登録成功");
                        self.state.hotkey_registered = true;
                    } else {
                        info!("ホットキー登録失敗");
                        self.state.hotkey_registered = false;
                    }
                }
            }
        }
    }
    fn recv_tasktray(&mut self) {
        if let Ok(event) = self.tray.rx.try_recv() {
            match event {
                endpoint::TrayEvent::ShowWindow => {
                    info!("タスクトレイから表示イベント受信");
                    // ウィンドウの表示/非表示切り替え
                    self.request_show_window();
                }
                endpoint::TrayEvent::Quit => {
                    info!("タスクトレイから終了イベント受信");
                    // 終了処理
                    // UIに終了を伝達
                    self.request_ui_exit();
                }
            }
        }
    }
    fn recv_ui(&mut self) {
        if let Ok(event) = self.ui.rx.try_recv() {
            info!("UIからのイベント受信");
            match event {
                endpoint::UiEvent::HwndReady(hwnd) => {
                    self.state.hwnd = Some(hwnd);

                    // hotkey に HWND を伝達
                    let _ = self.hotkey.tx.send(endpoint::HotkeyCmd::Register(hwnd));
                    // tasktray に HWND を伝達
                    // let _ = self.tray.tx.send(endpoint::TrayCmd::Register(hwnd));
                }
            }
        }
    }

    fn toggle_window(&mut self) {
        if let Some(hwnd) = self.state.hwnd {
            let is_visible = unsafe { IsWindowVisible(hwnd) };
            // info!("toggle_window: is_visible={}", is_visible);
            if is_visible != 0 {
                self.hide_window(hwnd);
                // info!("ウィンドウを非表示にしました");
            } else {
                // 表示
                self.show_window(hwnd);
                // info!("ウィンドウを表示しました");
            }
        } else {
            warn!("toggle_window: HWND が未設定です");
        }
    }
    fn request_show_window(&mut self) {
        if let Some(hwnd) = self.state.hwnd {
            let is_visible = unsafe { IsWindowVisible(hwnd) };
            if is_visible == 0 {
                self.show_window(hwnd);
            } else {
                info!("show_window: ウィンドウは既に表示されています");
            }
        } else {
            warn!("show_window: HWND が未設定です");
        }
    }

    fn request_ui_exit(&self) {
        let Some(hwnd) = self.state.hwnd else {
            warn!("HWND が未取得のため eframe を終了できません");
            return;
        };
        unsafe {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }

    fn show_window(&mut self, hwnd: HWND) {
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            let ok = SetForegroundWindow(hwnd);
            if ok == 0 {
                warn!(
                    "SetForegroundWindow が失敗しました（OS の制約で拒否される場合があります）: hwnd={:?}",
                    hwnd
                );
            }
        }
        let _ = self.ui.tx.send(endpoint::UiCommand::ForcusInput);
    }
    fn hide_window(&mut self, hwnd: HWND) {
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}
