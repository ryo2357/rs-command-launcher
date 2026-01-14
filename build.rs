use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let out_dir = env::var_os("OUT_DIR").expect("build.rs:OUT_DIR is not set");
    decode_tray_icon(&out_dir).expect("build.rs:falied to decode tray icon");
    Ok(())
}

fn decode_tray_icon(out_dir: &std::ffi::OsStr) -> anyhow::Result<()> {
    let dest_path = Path::new(out_dir).join("tray_icon_data.rs");

    // 画像をデコードして RGBA データを取得
    let img = image::open("assets/tray.png")
        .expect("Failed to open tray.png")
        .into_rgba8();
    let (w, h) = img.dimensions();
    let rgba = img.as_raw();
    let mut f = File::create(&dest_path)?;
    writeln!(f, "pub const TRAY_WIDTH: u32 = {};", w)?;
    writeln!(f, "pub const TRAY_HEIGHT: u32 = {};", h)?;
    writeln!(f, "pub const TRAY_RGBA: &[u8] = &{:?};", rgba)?;

    // Cargo に再ビルドの条件を通知
    println!("cargo:rerun-if-changed=assets/tray.png");
    Ok(())
}
