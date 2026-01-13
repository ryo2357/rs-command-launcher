use eframe::egui;
use log::{debug, error, info};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
// use std::sync::mpsc;
// use std::time::Duration;
use windows_sys::Win32::Foundation::HWND;
// use windows_sys::Win32::UI::WindowsAndMessaging::{
//     IsWindowVisible, PostMessageW, SW_HIDE, SW_SHOW, SetForegroundWindow, ShowWindow, WM_CLOSE,
// };
use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};

use crate::config::Settings;
use crate::model::commands;
use crate::runner;

use crate::app::endpoint::{UiCommand, UiEndpoint, UiEvent};

// use super::hotkey::HotkeyToggle;
// use super::task_tray::{TaskTray, TrayCommand};

#[derive(PartialEq)]
enum InitState {
    Start,
    Ready,
}
pub struct Launcher {
    state: InitState,
    command_input: String,
    commands: commands::Commands,
    hwnd: Option<HWND>,

    endpoint: UiEndpoint,
    // hotkey: Option<HotkeyToggle>,
    // // タスクトレイ機能
    // _tray: TaskTray,
    // tray_rx: mpsc::Receiver<TrayCommand>,
    // egui_ctx: egui::Context,
}

impl Launcher {
    pub fn new(settings: Settings, endpoint: UiEndpoint) -> anyhow::Result<Self> {
        let commands = settings.commands();
        Ok(Self {
            state: InitState::Start,
            command_input: String::new(),
            commands,
            hwnd: None,
            endpoint,
        })
    }

    // 初期化処理
    // HWNDの取得。伝達
    fn ensure_initialised(&mut self, frame: &mut eframe::Frame) {
        self.ensure_hwnd(frame);
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
            let _ = self.endpoint.tx.send(UiEvent::HwndReady(hwnd));
            log::info!("HWND 取得完了: {:?}", hwnd);
        }
    }

    // コントローラーからのイベント受信処理
    fn process_controller(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(cmd) = self.endpoint.rx.try_recv() {
            // match cmd {
            //     UiCommand::HideWindow => {
            //         self.hide_window();
            //         info!("ウィンドウを非表示にしました");
            //     }
            // }
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

impl eframe::App for Launcher {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 初期化処理
        if self.state == InitState::Start {
            debug!("LauncherApp: 初期化処理を開始します");
            // 初期化が必要な場合はここで行う
            self.ensure_initialised(frame);
            debug!("LauncherApp: 初期化処理が完了しました");
        }

        // コントローラーからのイベント処理
        self.process_controller(ctx, frame);

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
    }
}
