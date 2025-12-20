mod config;
mod paths;
mod runner;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let settings_path = paths::settings_path()?;
    let env_path = paths::env_path()?;

    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("paths") => {
            println!("settings: {}", settings_path.display());
            println!("env: {}", env_path.display());
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
            tracing::info!(name = %first.name, "起動しました");
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
            tracing::info!(name = %cmd.name, "起動しました");
            return Ok(());
        }
        _ => {
            let settings = config::load_settings(&settings_path)?;
            let _env_vars = config::load_env_vars(&env_path)?;
            tracing::info!(commands = settings.commands.len(), "設定を読み込みました");
        }
    }

    Ok(())
}
