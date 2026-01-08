use log::info;

use crate::config::Settings;

use super::hotkey::HotkeyToggle;
use super::initialize;
use super::launcher::LauncherApp;

pub fn run(settings: Settings) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions::default();

    // ホットキースレッドの起動
    let hotkey: Option<HotkeyToggle> = None;

    info!("UI を起動します");

    eframe::run_native(
        "command-launcher",
        native_options,
        Box::new(|cc| {
            initialize::initialize(&cc.egui_ctx);

            Ok(Box::new(LauncherApp::new(settings, hotkey)?))
        }),
    )
    .map_err(|e| anyhow::Error::msg(format!("UI を起動できません: {e:?}")))?;

    info!("UI を終了しました");
    Ok(())
}
