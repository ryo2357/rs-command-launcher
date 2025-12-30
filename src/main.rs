use anyhow::Context;
use log::LevelFilter;
use log::{error, info};

mod config;
mod model;
mod runner;

// mod ui;
fn main() {
    init_logger();

    match app() {
        Ok(_) => {}
        Err(e) => {
            error!("エラー: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn init_logger() {
    // ログ初期化
    // DebugビルドならDEBUGレベル、ReleaseビルドならINGOログ出力しない
    let is_debug = cfg!(debug_assertions);
    env_logger::Builder::new()
        .filter_level(if is_debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init();
}

fn app() -> anyhow::Result<()> {
    let commands = config::load_settings()?;

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("list") => {
            info!("commands list: {:?}", commands);
            return Ok(());
        }
        Some("run-first") => {
            let first = commands.first().context("commands が空です")?;

            runner::spawn_command(first)?;
            info!("{:?}を起動しました", first.name());
            return Ok(());
        }
        Some("run") => {
            let name = args.get(2).context("使い方: command-launcher run <name>")?;

            let cmd = commands
                .find_by_name(name)
                .with_context(|| format!("指定されたコマンドが見つかりません: {name}"))?;

            runner::spawn_command(cmd)?;
            info!("{:?}を起動しました", cmd.name());
            return Ok(());
        }
        _ => {

            // ui::run()?;
        }
    }
    Ok(())
}
