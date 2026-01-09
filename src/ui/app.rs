use eframe::egui;
use log::{debug, error, info};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::sync::mpsc;
use std::time::Duration;
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsWindowVisible, PostMessageW, SW_HIDE, SW_SHOW, SetForegroundWindow, ShowWindow, WM_CLOSE,
};

use crate::config::Settings;
use crate::model::commands;
use crate::runner;

use super::hotkey::HotkeyToggle;
use super::task_tray::{TaskTray, TrayCommand};

#[derive(PartialEq)]
enum InitState {
    Start,
    Ready,
}
pub struct LauncherApp {
    state: InitState,
    command_input: String,
    commands: commands::Commands,
    hwnd: Option<HWND>,
    hotkey: Option<HotkeyToggle>,
    // タスクトレイ機能
    _tray: TaskTray,
    tray_rx: mpsc::Receiver<TrayCommand>,
    egui_ctx: egui::Context,
}

impl LauncherApp {
    pub fn new(
        settings: Settings,
        egui_ctx: egui::Context,
        hotkey: Option<HotkeyToggle>,
        tray: TaskTray,
        tray_rx: mpsc::Receiver<TrayCommand>,
    ) -> anyhow::Result<Self> {
        let commands = settings.commands();
        Ok(Self {
            state: InitState::Start,
            command_input: String::new(),
            commands,
            hwnd: None,
            hotkey,
            _tray: tray,
            tray_rx,
            egui_ctx,
        })
    }

    fn ensure_initialised(&mut self, frame: &mut eframe::Frame) {
        self.ensure_hwnd(frame);
        self.ensure_hotkey();
        self.ensure_tray();
        self.state = InitState::Ready;
    }
    fn ensure_hwnd(&mut self, frame: &mut eframe::Frame) {
        if self.hwnd.is_some() {
            return;
        }

        if let Ok(handle) = frame.window_handle()
            && let RawWindowHandle::Win32(h) = handle.as_raw()
        {
            let hwnd = h.hwnd.get();
            self.hwnd = Some(hwnd);
            // トレイ側からの WM_NULL 起床が効くように、HWND は早めに注入する
            self._tray.set_hwnd(hwnd);
            log::info!("HWND 取得完了: {:?}", hwnd);
        }
    }
    fn ensure_hotkey(&mut self) {
        if self.hotkey.is_some() {
            error!("ホットキー登録失敗: 既にホットキーが登録されています");
            return;
        }

        let Some(hwnd) = self.hwnd else {
            error!("ホットキー登録失敗: HWND が未設定です");
            return;
        };

        match HotkeyToggle::register(hwnd, self.egui_ctx.clone()) {
            Ok(hk) => {
                self.hotkey = Some(hk);
                info!("ホットキー登録完了");
            }
            Err(e) => {
                error!("ホットキー登録失敗: {e:?}");
            }
        }
    }

    fn ensure_tray(&mut self) {
        if let Some(hwnd) = self.hwnd {
            self._tray.set_hwnd(hwnd);
        } else {
            error!("タスクトレイ初期化失敗: HWND が未設定です");
        }
    }

    fn show_window(&self) {
        let Some(hwnd) = self.hwnd else {
            error!("show_window: HWND が未設定です");
            return;
        };
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            SetForegroundWindow(hwnd);
        }
    }

    fn hide_window(&self) {
        let Some(hwnd) = self.hwnd else {
            error!("hide_window: HWND が未設定です");
            return;
        };
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }

        // ウィンドウ非表示中は eframe/egui の update が止まりがちなので、
        // まず一度だけ将来の再描画を予約して、受信ポーリングが再開するきっかけを作る。
        self.egui_ctx
            .request_repaint_after(Duration::from_millis(50));
    }
    fn is_window_visible(&self) -> Option<bool> {
        let hwnd = self.hwnd?;
        // 0 = false, non-0 = true
        Some(unsafe { IsWindowVisible(hwnd) != 0 })
    }

    fn toggle_window(&mut self, frame: &mut eframe::Frame) {
        if self.hwnd.is_none() {
            self.ensure_hwnd(frame);
        }

        match self.is_window_visible() {
            Some(true) => self.hide_window(),
            Some(false) => self.show_window(),
            None => error!("toggle_window: HWND が未設定です"),
        }
    }
    // タスクトレイからのコマンド処理
    fn process_tray_commands(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.hwnd.is_none() {
            self.ensure_hwnd(frame);
        }

        while let Ok(cmd) = self.tray_rx.try_recv() {
            match cmd {
                TrayCommand::Show => {
                    info!("tray: show");
                    if self.is_window_visible() == Some(false) {
                        self.show_window();
                    }
                }
                TrayCommand::Quit => {
                    info!("tray: quit");
                    // 非表示中でも確実に終了させるため、Win32 に WM_CLOSE を投げる
                    if let Some(hwnd) = self.hwnd {
                        unsafe {
                            PostMessageW(hwnd, WM_CLOSE, 0, 0);
                        }
                    } else {
                        // 念のため（HWND 未取得時のフォールバック）
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            }
        }
    }
    // ホットキー処理
    fn process_hotkey(&mut self, frame: &mut eframe::Frame) {
        // 借用衝突を避けつつ、取りこぼしなく処理する
        loop {
            let fired = self.hotkey.as_ref().is_some_and(|hk| hk.try_recv_toggle());
            if !fired {
                break;
            }
            info!("hotkey: toggle");
            self.toggle_window(frame);
        }
    }

    // コマンド実行機能

    fn try_run_command(&mut self) {
        let input = self.command_input.trim();
        if input.is_empty() {
            info!("空のコマンド名が入力されました");
            return;
        }

        let Some(command) = self.commands.find_by_name(input) else {
            info!("指定されたコマンドが見つかりません: {:?}", input);
            self.command_input.clear();
            return;
        };

        match runner::spawn_command(command) {
            Ok(_child) => {
                info!("{:?}を起動しました", command.name());
                self.command_input.clear();
            }
            Err(e) => {
                info!("起動に失敗しました: {e:?}");
                self.command_input.clear();
            }
        }
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 初期化処理
        if self.state == InitState::Start {
            debug!("LauncherApp: 初期化処理を開始します");
            // 初期化が必要な場合はここで行う
            self.ensure_initialised(frame);
            debug!("LauncherApp: 初期化処理が完了しました");
        }

        // タスクトレイトUI処理
        self.process_tray_commands(ctx, frame);
        // ホットキーの処理
        self.process_hotkey(frame);

        // メインUI
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.command_input)
                    .hint_text("コマンド名を入力して Enter で実行")
                    .desired_width(f32::INFINITY),
            );

            // Enter キー（英字入力）または IME の確定で実行されるようにする
            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let lost = response.lost_focus();

            // Enter 押下（英字）またはフォーカス喪失 / テキスト変更のいずれかで実行
            if lost && enter_pressed {
                info!("入力確定で実行します");
                self.try_run_command();
            }

            ui.separator();
            // ui.label(&self.status);
        });

        // 非表示中でも tray/hotkey の受信処理を回すため、定期的に update を起こす。
        if self.is_window_visible() == Some(false) {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}
