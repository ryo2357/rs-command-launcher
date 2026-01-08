use eframe::egui;
use log::{debug, error, info};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::HWND;

use crate::config::Settings;
use crate::model::commands;
use crate::runner;

use super::hotkey::HotkeyToggle;

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
}

impl LauncherApp {
    pub fn new(settings: Settings, hotkey: Option<HotkeyToggle>) -> anyhow::Result<Self> {
        let commands = settings.commands();
        Ok(Self {
            state: InitState::Start,
            command_input: String::new(),
            commands,
            hwnd: None,
            hotkey,
        })
    }

    fn ensure_initialised(&mut self, frame: &mut eframe::Frame) {
        self.ensure_hwnd(frame);
        self.ensure_hotkey();
        self.state = InitState::Ready;
    }
    fn ensure_hwnd(&mut self, frame: &mut eframe::Frame) {
        if self.hwnd.is_some() {
            return;
        }

        if let Ok(handle) = frame.window_handle() {
            if let RawWindowHandle::Win32(h) = handle.as_raw() {
                let hwnd = h.hwnd.get();
                self.hwnd = Some(hwnd);
                log::info!("HWND 取得完了: {:?}", hwnd);
            }
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

        match HotkeyToggle::register(hwnd) {
            Ok(hk) => {
                self.hotkey = Some(hk);
                info!("ホットキー登録完了");
            }
            Err(e) => {
                error!("ホットキー登録失敗: {e:?}");
            }
        }
    }

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

        // ホットキーでウィンドウ表示/非表示の切り替え
        if self.hwnd.is_none() {
            self.hwnd = get_hwnd(frame);
        }
    }
}

pub fn get_hwnd(frame: &eframe::Frame) -> Option<HWND> {
    let window = frame.window_handle();
    match window.ok()?.as_raw() {
        RawWindowHandle::Win32(h) => Some(h.hwnd.get()),
        _ => None,
    }
}
