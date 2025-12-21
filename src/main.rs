use log::LevelFilter;
use anyhow::Context;
use log::{info,error};

mod config;
mod paths;
mod runner;
fn main()  {

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
    // DebugビルドならInfoレベル、Releaseビルドならログ出力しない
    let is_debug = cfg!(debug_assertions);
    env_logger::Builder::new()
        .filter_level(if is_debug { LevelFilter::Info } else { LevelFilter::Off })
        .init();
}

fn app() -> anyhow::Result<()> {
    let settings_path = paths::settings_path()?;
    let env_path = paths::env_path()?;

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("paths") => {
            info!("settings: {}", settings_path.display());
            info!("env: {}", env_path.display());
            return Ok(());
        }
        Some("run-first") => {
            let settings = config::load_settings(&settings_path)?;
            let env_vars = config::load_env_vars(&env_path)?;
            let first = settings
                .commands
                .first()
                .context("commands が空です")?;

            runner::spawn_command(first, &env_vars)?;
            info!("{:?}を起動しました", first.name);
            return Ok(());
        }
        Some("run") => {
            let name = args
                .get(2)
                .context("使い方: command-launcher run <name>")?;
            let settings = config::load_settings(&settings_path)?;
            let env_vars = config::load_env_vars(&env_path)?;

            let cmd = settings
                .commands
                .iter()
                .find(|c| c.name == *name)
                .with_context(|| format!("指定されたコマンドが見つかりません: {name}"))?;

            runner::spawn_command(cmd, &env_vars)?;
            info!("{:?}を起動しました", cmd.name);
            return Ok(());
        }
        _ => {
            let settings = config::load_settings(&settings_path)?;
            let _env_vars = config::load_env_vars(&env_path)?;
            info!("設定を読み込みました: {} 件のコマンド", settings.commands.len());
        }
    }
  Ok(())
}