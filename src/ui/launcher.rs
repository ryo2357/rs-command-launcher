use eframe::egui;
use log::{debug, error, info};

use crate::config::Settings;
use crate::model::commands;
use crate::runner;
use crate::ui::initialize;

pub fn run(settings: Settings) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions::default();
    info!("UI を起動します");

    eframe::run_native(
        "command-launcher",
        native_options,
        Box::new(|cc| {
            initialize::initialize(&cc.egui_ctx);
            Ok(Box::new(LauncherApp::new(settings)?))
        }),
    )
    .map_err(|e| anyhow::Error::msg(format!("UI を起動できません: {e:?}")))?;

    info!("UI を終了しました");
    Ok(())
}

enum LauncherStatus {
    Hiddlen,
    Ready,
}
struct LauncherApp {
    command_input: String,
    status: LauncherStatus,
    commands: commands::Commands,
}

impl LauncherApp {
    fn new(settings: Settings) -> anyhow::Result<Self> {
        let commands = settings.commands();
        Ok(Self {
            command_input: String::new(),
            status: LauncherStatus::Ready,
            commands,
        })
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
            if (lost && enter_pressed) {
                info!("入力確定で実行します");
                self.try_run_command();
            }

            ui.separator();
            // ui.label(&self.status);
        });
    }
}
