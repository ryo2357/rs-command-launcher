use eframe::egui;
use log::{error, info};

use crate::{config, paths, runner};

pub fn run() -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "command-launcher",
        native_options,
        Box::new(|_cc| Ok(Box::new(LauncherApp::new()?))),
    )
    .map_err(|e| anyhow::Error::msg(format!("UI を起動できません: {e:?}")))?;

    Ok(())
}

struct LauncherApp {
    command_input: String,
    status: String,
    settings: Option<config::Settings>,
    env_vars: Option<config::EnvVars>,
}

impl LauncherApp {
    fn new() -> anyhow::Result<Self> {
        let (settings, env_vars, status) = match load_config() {
            Ok((settings, env_vars)) => {
                let status = format!("設定を読み込みました: {} 件のコマンド", settings.commands.len());
                (Some(settings), Some(env_vars), status)
            }
            Err(e) => {
                error!("設定読み込みに失敗しました: {e:?}");
                (None, None, format!("設定読み込みに失敗しました: {e:#}"))
            }
        };

        Ok(Self {
            command_input: String::new(),
            status,
            settings,
            env_vars,
        })
    }

    fn try_run_command(&mut self) {
        let input = self.command_input.trim();
        if input.is_empty() {
            self.status = "コマンド名を入力してください".to_string();
            return;
        }

        let Some(settings) = self.settings.as_ref() else {
            self.status = "設定が読み込まれていません".to_string();
            return;
        };
        let Some(env_vars) = self.env_vars.as_ref() else {
            self.status = "環境変数が読み込まれていません".to_string();
            return;
        };

        let Some(command) = settings.commands.iter().find(|c| c.name == input) else {
            self.status = format!("指定されたコマンドが見つかりません: {input}");
            return;
        };

        match runner::spawn_command(command, env_vars) {
            Ok(_child) => {
                info!("{:?}を起動しました", command.name);
                self.status = format!("起動しました: {}", command.name);
                self.command_input.clear();
            }
            Err(e) => {
                error!("起動に失敗しました: {e:?}");
                self.status = format!("起動に失敗しました: {e:#}");
            }
        }
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("コマンド名を入力して Enter で実行");

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.command_input)
                    .hint_text("CommandSpec.name")
                    .desired_width(f32::INFINITY),
            );

            if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.try_run_command();
            }

            ui.separator();
            ui.label(&self.status);
        });
    }
}

fn load_config() -> anyhow::Result<(config::Settings, config::EnvVars)> {
    let settings_path = paths::settings_path()?;
    let env_path = paths::env_path()?;

    let settings = config::load_settings(&settings_path)?;
    let env_vars = config::load_env_vars(&env_path)?;

    Ok((settings, env_vars))
}
