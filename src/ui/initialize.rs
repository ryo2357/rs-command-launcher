use eframe::egui;

const FONT: &[u8] = include_bytes!(r"C:/Windows/Fonts/MEIRYO.TTC");

pub fn initialize(ctx: &egui::Context) {
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
