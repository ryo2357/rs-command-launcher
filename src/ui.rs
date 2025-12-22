use eframe::egui;
use log::{error, info};

use crate::runner;
use crate::config::{Settings, EnvVars};

pub fn run(settings: Settings, env_vars: EnvVars) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "command-launcher",
        native_options,
        Box::new(|_cc| Ok(Box::new(LauncherApp::new(settings,env_vars)?))),
    )
    .map_err(|e| anyhow::Error::msg(format!("UI を起動できません: {e:?}")))?;

    Ok(())
}

struct LauncherApp {
    command_input: String,
    status: String,
    settings: Option<Settings>,
    env_vars: Option<EnvVars>,
}

impl LauncherApp {
    fn new(settings: Settings, env_vars: EnvVars) -> anyhow::Result<Self> {
        Ok(Self {
            command_input: String::new(),
            status: format!("設定を読み込みました: {} 件のコマンド", settings.commands.len()),
            settings: Some(settings),
            env_vars: Some(env_vars),
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
