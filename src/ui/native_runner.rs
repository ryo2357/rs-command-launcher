use eframe::egui;
use log::info;

use crate::config::Settings;
use crate::ui::Launcher;

use crate::app::endpoint::UiEndpoint;

const FONT: &[u8] = include_bytes!(r"C:/Windows/Fonts/MEIRYO.TTC");

pub fn eframe_startup(settings: Settings, ui_endpoint: UiEndpoint) -> anyhow::Result<()> {
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "command-launcher",
        native_options,
        Box::new(move |cc| {
            initialize(&cc.egui_ctx);

            Ok(Box::new(Launcher::new(settings, ui_endpoint)?))
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
