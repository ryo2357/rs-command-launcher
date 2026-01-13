use log::{info, warn};
use std::sync::mpsc;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{IsWindowVisible, SW_HIDE, SW_SHOW, ShowWindow};

use crate::app::endpoint;

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
            if let Ok(_) = self.finish_rx.try_recv() {
                // // ドロップトレイトからの処理だとうまくいかないのでここで明示的に終了処理を行う
                // info!("コントローラーの終了処理");
                let _ = self.hotkey.tx.send(endpoint::HotkeyCmd::Finish);
                break;
            }
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
    fn recv_tasktray(&mut self) {}
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
                unsafe {
                    ShowWindow(hwnd, SW_HIDE);
                }
                // info!("ウィンドウを非表示にしました");
            } else {
                // 表示
                unsafe {
                    ShowWindow(hwnd, SW_SHOW);
                }
                // info!("ウィンドウを表示しました");
            }
        } else {
            warn!("toggle_window: HWND が未設定です");
        }
    }
}
