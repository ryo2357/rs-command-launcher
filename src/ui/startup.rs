use eframe::egui;
use log::info;

use crate::config::Settings;

use super::app::LauncherApp;
use super::hotkey::HotkeyToggle;
use super::task_tray::TaskTray;

const FONT: &[u8] = include_bytes!(r"C:/Windows/Fonts/MEIRYO.TTC");

pub fn startup(settings: Settings) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions::default();

    // ホットキースレッドの起動
    let hotkey: Option<HotkeyToggle> = None;
    // タスクトレイ生成
    let (tray, tray_rx) = TaskTray::new()?;

    info!("UI を起動します");

    eframe::run_native(
        "command-launcher",
        native_options,
        Box::new(move |cc| {
            initialize(&cc.egui_ctx);

            tray.set_ctx(cc.egui_ctx.clone());

            Ok(Box::new(LauncherApp::new(
                settings,
                cc.egui_ctx.clone(),
                hotkey,
                tray,
                tray_rx,
            )?))
        }),
    )
    .map_err(|e| anyhow::Error::msg(format!("UI を起動できません: {e:?}")))?;

    info!("UI を終了しました");
    Ok(())
}

fn initialize(ctx: &egui::Context) {
    configure_fonts(ctx);
}

fn configure_fonts(ctx: &egui::Context) {
    // 日本語フォントをプロジェクトに追加してからパスを合わせてください
    // 例: assets/fonts/NotoSansJP-Regular.ttf

    let mut fonts = egui::FontDefinitions::default();

    fonts
        .font_data
        .insert("jp".to_owned(), egui::FontData::from_static(FONT).into());

    // 優先順位: Proportional / Monospace の先頭に jp を入れてフォールバックさせる
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "jp".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "jp".to_owned());

    ctx.set_fonts(fonts);
}
